use chrono::prelude::*;
use diesel;
use diesel::dsl::{self, select, sql};
use diesel::prelude::*;
use diesel::sql_types::{Bigint, Int4, Nullable, Uuid as dUuid};
use models::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use schema::{comps, holds};
use std::borrow::Cow;
use utils::errors::*;
use uuid::Uuid;
use validator::*;
use validators::{self, *};

sql_function!(fn comps_quantity_valid_for_hold_quantity(hold_id: dUuid, id: dUuid, quantity: Int4) -> Bool);
sql_function!(fn comps_hold_type_valid_for_comp_creation(hold_id: dUuid) -> Bool);

#[derive(Debug, Deserialize, Identifiable, PartialEq, Queryable, Serialize)]
pub struct Comp {
    pub id: Uuid,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub hold_id: Uuid,
    pub quantity: i32,
    pub redemption_code: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default, Deserialize, Validate)]
#[table_name = "comps"]
pub struct UpdateCompAttributes {
    pub name: Option<String>,
    #[validate(email(message = "Email is invalid"))]
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub email: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub phone: Option<String>,
    pub quantity: Option<i32>,
}

impl Comp {
    pub fn find_for_hold(hold_id: Uuid, conn: &PgConnection) -> Result<Vec<Comp>, DatabaseError> {
        comps::table
            .inner_join(holds::table.on(holds::id.eq(comps::hold_id)))
            .filter(holds::hold_type.eq(HoldTypes::Comp.to_string()))
            .filter(comps::hold_id.eq(hold_id))
            .order_by(comps::name)
            .select(comps::all_columns)
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to retrieve comps for hold")
    }

    pub fn find_by_redemption_code(
        redemption_code: &str,
        conn: &PgConnection,
    ) -> Result<Comp, DatabaseError> {
        comps::table
            .filter(comps::redemption_code.eq(redemption_code.to_uppercase()))
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load comp with that redeem code",
            )
    }

    pub fn sum_for_hold(hold_id: Uuid, conn: &PgConnection) -> Result<u32, DatabaseError> {
        comps::table
            .inner_join(holds::table.on(holds::id.eq(comps::hold_id)))
            .filter(holds::hold_type.eq(HoldTypes::Comp.to_string()))
            .filter(comps::hold_id.eq(hold_id))
            .select(sql::<Nullable<Bigint>>("sum(quantity)"))
            .first::<Option<i64>>(conn)
            .map(|n| n.unwrap_or(0) as u32)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to retrieve comp sum for hold",
            )
    }

    pub fn create(
        name: String,
        hold_id: Uuid,
        email: Option<String>,
        phone: Option<String>,
        quantity: u16,
    ) -> NewComp {
        let redemption_code = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>()
            .to_uppercase();
        NewComp {
            name,
            hold_id,
            email,
            phone,
            quantity: quantity as i32,
            redemption_code,
        }
    }

    fn validate_record(
        &self,
        update_attrs: &UpdateCompAttributes,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let mut validation_errors = update_attrs.validate();

        validation_errors = validators::append_validation_error(
            validation_errors,
            "quantity",
            Comp::comps_quantity_valid_for_hold_quantity(
                self.hold_id,
                Some(self.id),
                update_attrs.quantity.unwrap_or(self.quantity),
                conn,
            )?,
        );

        Ok(validation_errors?)
    }

    pub fn update(
        &self,
        update_attrs: UpdateCompAttributes,
        conn: &PgConnection,
    ) -> Result<Comp, DatabaseError> {
        self.validate_record(&update_attrs, conn)?;

        diesel::update(
            comps::table
                .filter(comps::id.eq(self.id))
                .filter(comps::updated_at.eq(self.updated_at)),
        ).set((update_attrs, comps::updated_at.eq(dsl::now)))
        .get_result(conn)
        .to_db_error(ErrorCode::UpdateError, "Could not update comp")
    }

    pub fn comps_quantity_valid_for_hold_quantity(
        hold_id: Uuid,
        id: Option<Uuid>,
        quantity: i32,
        conn: &PgConnection,
    ) -> Result<Result<(), ValidationError>, DatabaseError> {
        let result = select(comps_quantity_valid_for_hold_quantity(
            hold_id,
            id.unwrap_or(Uuid::default()),
            quantity,
        )).get_result::<bool>(conn)
        .to_db_error(
            if id.is_none() {
                ErrorCode::InsertError
            } else {
                ErrorCode::UpdateError
            },
            "Could not confirm if comp quantity valid for hold",
        )?;
        if !result {
            let mut validation_error = create_validation_error(
                "comps_quantity_valid_for_hold_quantity",
                "Comp quantity is too large for hold quantity",
            );
            validation_error.add_param(Cow::from("id"), &id);
            validation_error.add_param(Cow::from("hold_id"), &hold_id);
            validation_error.add_param(Cow::from("quantity"), &quantity);

            return Ok(Err(validation_error));
        }
        Ok(Ok(()))
    }

    pub fn comps_hold_type_valid_for_comp_creation(
        hold_id: Uuid,
        id: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Result<(), ValidationError>, DatabaseError> {
        let result = select(comps_hold_type_valid_for_comp_creation(hold_id))
            .get_result::<bool>(conn)
            .to_db_error(
                if id.is_none() {
                    ErrorCode::InsertError
                } else {
                    ErrorCode::UpdateError
                },
                "Could not confirm if comp valid for hold type",
            )?;
        if !result {
            let mut validation_error = create_validation_error(
                "comps_hold_type_valid_for_comp_creation",
                "Comps can only be associated with holds that have Comp hold type",
            );
            validation_error.add_param(Cow::from("id"), &id);
            validation_error.add_param(Cow::from("hold_id"), &hold_id);

            return Ok(Err(validation_error));
        }
        Ok(Ok(()))
    }

    pub fn find(hold_id: Uuid, id: Uuid, conn: &PgConnection) -> Result<Comp, DatabaseError> {
        comps::table
            .filter(comps::hold_id.eq(hold_id))
            .filter(comps::id.eq(id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve comp")
    }

    pub fn destroy(&self, conn: &PgConnection) -> Result<usize, DatabaseError> {
        // TODO: prevent deletion of comps that have been claimed
        DatabaseError::wrap(
            ErrorCode::DeleteError,
            "Could not remove comp",
            diesel::delete(self).execute(conn),
        )
    }

    pub fn destroy_from_hold(hold_id: Uuid, conn: &PgConnection) -> Result<usize, DatabaseError> {
        // TODO: prevent deletion of comps that have been claimed
        DatabaseError::wrap(
            ErrorCode::DeleteError,
            "Could not remove comps",
            diesel::delete(comps::table.filter(comps::hold_id.eq(hold_id))).execute(conn),
        )
    }
}

#[derive(Deserialize, Insertable, Serialize, Validate)]
#[table_name = "comps"]
pub struct NewComp {
    pub name: String,
    pub hold_id: Uuid,
    #[validate(email(message = "Email is invalid"))]
    pub email: Option<String>,
    pub phone: Option<String>,
    pub quantity: i32,
    pub redemption_code: String,
}

impl NewComp {
    pub fn commit(self, conn: &PgConnection) -> Result<Comp, DatabaseError> {
        self.validate_record(conn)?;

        diesel::insert_into(comps::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create comp")
    }

    fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let mut validation_errors = self.validate();

        validation_errors = validators::append_validation_error(
            validation_errors,
            "quantity",
            Comp::comps_quantity_valid_for_hold_quantity(self.hold_id, None, self.quantity, conn)?,
        );
        validation_errors = validators::append_validation_error(
            validation_errors,
            "hold_id",
            Comp::comps_hold_type_valid_for_comp_creation(self.hold_id, None, conn)?,
        );

        Ok(validation_errors?)
    }
}
