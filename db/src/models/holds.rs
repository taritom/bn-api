use chrono::prelude::*;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::holds;
use utils::errors::*;
use uuid::Uuid;
use validator::*;
use validators::{self, *};

#[derive(Deserialize, Identifiable, Queryable, Serialize, PartialEq, Debug)]
pub struct Hold {
    pub id: Uuid,
    pub name: String,
    pub event_id: Uuid,
    pub redemption_code: String,
    pub discount_in_cents: Option<i64>,
    pub end_at: Option<NaiveDateTime>,
    pub max_per_order: Option<i64>,
    pub hold_type: String,
    pub ticket_type_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default)]
#[table_name = "holds"]
pub struct UpdateHoldAttributes {
    pub name: Option<String>,
    pub redemption_code: Option<String>,
    pub hold_type: Option<String>,
    pub discount_in_cents: Option<Option<i64>>,
    pub end_at: Option<Option<NaiveDateTime>>,
    pub max_per_order: Option<Option<i64>>,
}

impl Hold {
    pub fn create(
        name: String,
        event_id: Uuid,
        redemption_code: String,
        discount_in_cents: Option<u32>,
        end_at: Option<NaiveDateTime>,
        max_per_order: Option<u32>,
        hold_type: HoldTypes,
        ticket_type_id: Uuid,
    ) -> NewHold {
        NewHold {
            name,
            event_id,
            redemption_code: redemption_code.to_uppercase(),
            discount_in_cents: discount_in_cents.and_then(|discount| Some(discount as i64)),
            end_at,
            max_per_order: max_per_order.map(|m| m as i64),
            hold_type: hold_type.to_string(),
            ticket_type_id,
        }
    }

    pub fn update(
        &self,
        mut update_attrs: UpdateHoldAttributes,
        conn: &PgConnection,
    ) -> Result<Hold, DatabaseError> {
        if update_attrs.hold_type == Some(HoldTypes::Comp.to_string()) {
            update_attrs.discount_in_cents = Some(None);
        }

        self.validate_record(&update_attrs, conn)?;
        diesel::update(
            holds::table
                .filter(holds::id.eq(self.id))
                .filter(holds::updated_at.eq(self.updated_at)),
        ).set((update_attrs, holds::updated_at.eq(dsl::now)))
        .get_result(conn)
        .to_db_error(ErrorCode::UpdateError, "Could not update hold")
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<Hold, DatabaseError> {
        holds::table
            .filter(holds::id.eq(id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve hold")
    }

    pub fn find_for_event(event_id: Uuid, conn: &PgConnection) -> Result<Vec<Hold>, DatabaseError> {
        holds::table
            .filter(holds::event_id.eq(event_id))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve holds for event")
    }

    fn validate_record(
        &self,
        update_attrs: &UpdateHoldAttributes,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let discount_in_cents_valid = Hold::discount_in_cents_valid(
            update_attrs
                .hold_type
                .clone()
                .unwrap_or(self.hold_type.clone()),
            if update_attrs.discount_in_cents.is_some() {
                update_attrs.discount_in_cents.unwrap()
            } else {
                self.discount_in_cents
            },
        );
        let unique_redemption_code = redemption_code_unique_per_event_validation(
            Some(self.id),
            "holds".into(),
            update_attrs
                .redemption_code
                .clone()
                .unwrap_or(self.redemption_code.clone()),
            conn,
        )?;

        let validation_errors = validators::append_validation_error(
            Ok(()),
            "discount_in_cents",
            discount_in_cents_valid,
        );
        let validation_errors = validators::append_validation_error(
            validation_errors,
            "redemption_code",
            unique_redemption_code,
        );

        Ok(validation_errors?)
    }

    pub fn split(
        &self,
        name: String,
        redemption_code: String,
        quantity: u32,
        discount_in_cents: Option<u32>,
        hold_type: HoldTypes,
        end_at: Option<NaiveDateTime>,
        max_per_order: Option<u32>,
        conn: &PgConnection,
    ) -> Result<Hold, DatabaseError> {
        let new_hold = Hold::create(
            name,
            self.event_id,
            redemption_code,
            discount_in_cents,
            end_at,
            max_per_order,
            hold_type,
            self.ticket_type_id,
        ).commit(conn)?;

        TicketInstance::add_to_hold(
            new_hold.id,
            self.ticket_type_id,
            quantity,
            Some(self.id),
            conn,
        )?;
        Ok(new_hold)
    }

    pub fn destroy(self, conn: &PgConnection) -> Result<(), DatabaseError> {
        self.set_quantity(0, conn)?;
        diesel::delete(holds::table.filter(holds::id.eq(self.id)))
            .execute(conn)
            .to_db_error(ErrorCode::DeleteError, "Could not delete hold")?;
        Ok(())
    }

    pub fn discount_in_cents_valid(
        hold_type: String,
        discount_in_cents: Option<i64>,
    ) -> Result<(), ValidationError> {
        if hold_type == HoldTypes::Discount.to_string() && discount_in_cents.is_none() {
            return Err(ValidationError::new("required"));
        }

        Ok(())
    }

    pub fn set_quantity(&self, quantity: u32, conn: &PgConnection) -> Result<(), DatabaseError> {
        // Validate logic is not releasing already assigned comps
        if self.hold_type == HoldTypes::Comp.to_string() && self.comps_sum(conn)? > quantity {
            validators::append_validation_error(
                Ok(()),
                "quantity",
                Err(ValidationError::new(
                    &"assigned_comp_count_greater_than_quantity",
                )),
            )?;
        }

        let count = self.quantity(conn)?;
        if count < quantity {
            TicketInstance::add_to_hold(
                self.id,
                self.ticket_type_id,
                quantity - count,
                None,
                conn,
            )?;
        }
        if count > quantity {
            TicketInstance::release_from_hold(
                self.id,
                self.ticket_type_id,
                count - quantity,
                conn,
            )?;
        }
        Ok(())
    }

    pub fn comps_sum(&self, conn: &PgConnection) -> Result<u32, DatabaseError> {
        if self.hold_type == HoldTypes::Comp.to_string() {
            Comp::sum_for_hold(self.id, conn)
        } else {
            Ok(0)
        }
    }

    pub fn comps(&self, conn: &PgConnection) -> Result<Vec<Comp>, DatabaseError> {
        if self.hold_type == HoldTypes::Comp.to_string() {
            Comp::find_for_hold(self.id, conn)
        } else {
            Err(DatabaseError::new(
                ErrorCode::InternalError,
                Some("Comps only exist for holds with Comp hold_type".to_string()),
            ))
        }
    }

    pub fn quantity(&self, conn: &PgConnection) -> Result<u32, DatabaseError> {
        TicketInstance::count_for_hold(self.id, self.ticket_type_id, conn)
    }

    pub fn organization(&self, conn: &PgConnection) -> Result<Organization, DatabaseError> {
        use schema::*;
        events::table
            .inner_join(organizations::table)
            .filter(events::id.eq(self.event_id))
            .select(organizations::all_columns)
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load organization for hold",
            )
    }

    pub fn find_by_redemption_code(
        redemption_code: &str,
        conn: &PgConnection,
    ) -> Result<Hold, DatabaseError> {
        holds::table
            .filter(holds::redemption_code.eq(redemption_code.to_uppercase()))
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load hold with that redeem key",
            )
    }
}

#[derive(Insertable)]
#[table_name = "holds"]
pub struct NewHold {
    pub name: String,
    pub event_id: Uuid,
    pub redemption_code: String,
    pub discount_in_cents: Option<i64>,
    pub end_at: Option<NaiveDateTime>,
    pub max_per_order: Option<i64>,
    pub hold_type: String,
    pub ticket_type_id: Uuid,
}

impl NewHold {
    pub fn commit(mut self, conn: &PgConnection) -> Result<Hold, DatabaseError> {
        if self.hold_type == HoldTypes::Comp.to_string() {
            self.discount_in_cents = None
        }
        self.validate_record(conn)?;
        diesel::insert_into(holds::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create hold")
    }

    fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let discount_in_cents_valid =
            Hold::discount_in_cents_valid(self.hold_type.clone(), self.discount_in_cents);
        let unique_redemption_code = redemption_code_unique_per_event_validation(
            None,
            "holds".into(),
            self.redemption_code.clone(),
            conn,
        )?;

        let validation_errors = validators::append_validation_error(
            Ok(()),
            "discount_in_cents",
            discount_in_cents_valid,
        );
        let validation_errors = validators::append_validation_error(
            validation_errors,
            "redemption_code",
            unique_redemption_code,
        );

        Ok(validation_errors?)
    }
}
