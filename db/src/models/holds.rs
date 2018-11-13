use chrono::prelude::*;
use diesel;
use diesel::dsl::{self, select};
use diesel::prelude::*;
use diesel::sql_types::{Text, Uuid as dUuid};
use models::*;
use schema::holds;
use std::borrow::Cow;
use utils::errors::{self, *};
use uuid::Uuid;
use validator::*;
use validators::{self, *};

sql_function!(fn hold_can_change_type(id: dUuid, hold_type: Text) -> Bool);

#[derive(Clone, Deserialize, Identifiable, Queryable, Serialize, PartialEq, Debug)]
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

    fn hold_has_comps_in_use(
        id: Uuid,
        hold_type: String,
        conn: &PgConnection,
    ) -> Result<Result<(), ValidationError>, DatabaseError> {
        if hold_type == HoldTypes::Comp.to_string() {
            return Ok(Ok(()));
        }

        let result = select(hold_can_change_type(id, hold_type))
            .get_result::<bool>(conn)
            .to_db_error(
                errors::ErrorCode::UpdateError,
                "Could not confirm if hold has used comps",
            )?;
        if !result {
            let mut validation_error =
                create_validation_error("comps_in_use", "Hold has comps used by order items");
            validation_error.add_param(Cow::from("hold_id"), &id);
            return Ok(Err(validation_error));
        }
        Ok(Ok(()))
    }

    pub fn update(
        &self,
        update_attrs: UpdateHoldAttributes,
        conn: &PgConnection,
    ) -> Result<Hold, DatabaseError> {
        let mut update_attrs = update_attrs;
        if update_attrs.hold_type == Some(HoldTypes::Comp.to_string()) {
            // Remove discount
            update_attrs.discount_in_cents = Some(None);
        }

        self.validate_record(&update_attrs, conn)?;
        if update_attrs.hold_type == Some(HoldTypes::Comp.to_string()) {
            // Passes validation so safe to destroy remaining unused comps
            Comp::destroy_from_hold(self.id, conn)?;
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
        let validation_errors = validators::append_validation_error(
            Ok(()),
            "hold_type",
            Hold::hold_has_comps_in_use(
                self.id,
                update_attrs
                    .hold_type
                    .clone()
                    .unwrap_or(self.hold_type.clone()),
                conn,
            )?,
        );
        let validation_errors = validators::append_validation_error(
            validation_errors,
            "discount_in_cents",
            Hold::discount_in_cents_valid(
                update_attrs
                    .hold_type
                    .clone()
                    .unwrap_or(self.hold_type.clone()),
                if update_attrs.discount_in_cents.is_some() {
                    update_attrs.discount_in_cents.unwrap()
                } else {
                    self.discount_in_cents
                },
            ),
        );
        let validation_errors = validators::append_validation_error(
            validation_errors,
            "redemption_code",
            redemption_code_unique_per_event_validation(
                Some(self.id),
                "holds".into(),
                update_attrs
                    .redemption_code
                    .clone()
                    .unwrap_or(self.redemption_code.clone()),
                conn,
            )?,
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
        // Validate if hold eligible for deletion
        validators::append_validation_error(
            Ok(()),
            "hold_type",
            Hold::hold_has_comps_in_use(self.id, "".to_string(), conn)?,
        )?;

        Comp::destroy_from_hold(self.id, conn)?;
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
            let validation_error =
                create_validation_error("required", "Discount required for hold type Discount");
            return Err(validation_error);
        }

        Ok(())
    }

    pub fn set_quantity(&self, quantity: u32, conn: &PgConnection) -> Result<(), DatabaseError> {
        // Validate logic is not releasing already assigned comps
        if self.hold_type == HoldTypes::Comp.to_string() && self.comps_sum(conn)? > quantity {
            let mut validation_error = create_validation_error(
                "assigned_comp_count_greater_than_quantity",
                "Existing comp total quantity greater than new quantity",
            );
            validation_error.add_param(Cow::from("hold_id"), &self.id);

            validators::append_validation_error(Ok(()), "quantity", Err(validation_error))?;
        }

        let (count, _available) = self.quantity(conn)?;
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

    pub fn quantity(&self, conn: &PgConnection) -> Result<(u32, u32), DatabaseError> {
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
        let validation_errors = validators::append_validation_error(
            Ok(()),
            "discount_in_cents",
            Hold::discount_in_cents_valid(self.hold_type.clone(), self.discount_in_cents),
        );
        let validation_errors = validators::append_validation_error(
            validation_errors,
            "redemption_code",
            redemption_code_unique_per_event_validation(
                None,
                "holds".into(),
                self.redemption_code.clone(),
                conn,
            )?,
        );

        Ok(validation_errors?)
    }
}
