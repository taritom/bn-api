use chrono::NaiveDateTime;
use diesel;
use diesel::dsl::select;
use diesel::prelude::*;
use diesel::sql_types::Uuid as dUuid;
use models::{Code, TicketType};
use schema::ticket_type_codes;
use std::borrow::Cow;
use utils::errors::*;
use uuid::Uuid;
use validator::*;
use validators;

sql_function!(fn ticket_type_code_ticket_type_id_valid(code_id: dUuid, ticket_type_id: dUuid) -> Bool);

#[derive(Associations, Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(TicketType)]
#[belongs_to(Code)]
#[table_name = "ticket_type_codes"]
pub struct TicketTypeCode {
    pub id: Uuid,
    pub ticket_type_id: Uuid,
    pub code_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "ticket_type_codes"]
pub struct NewTicketTypeCode {
    pub ticket_type_id: Uuid,
    pub code_id: Uuid,
}

impl NewTicketTypeCode {
    pub fn commit(&self, conn: &PgConnection) -> Result<TicketTypeCode, DatabaseError> {
        self.validate_record(conn)?;

        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not add code to ticket_type",
            diesel::insert_into(ticket_type_codes::table)
                .values(self)
                .get_result(conn),
        )
    }

    fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        Ok(validators::append_validation_error(
            Ok(()),
            "ticket_type_id",
            TicketTypeCode::ticket_type_code_ticket_type_id_valid(
                self.code_id,
                self.ticket_type_id,
                conn,
            )?,
        )?)
    }
}

impl TicketTypeCode {
    pub fn create(ticket_type_id: Uuid, code_id: Uuid) -> NewTicketTypeCode {
        NewTicketTypeCode {
            ticket_type_id,
            code_id,
        }
    }

    pub fn destroy_multiple(
        code_id: Uuid,
        ticket_type_ids: Vec<Uuid>,
        conn: &PgConnection,
    ) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::DeleteError,
            "Could not remove ticket type codes",
            diesel::delete(
                ticket_type_codes::table
                    .filter(ticket_type_codes::code_id.eq(code_id))
                    .filter(ticket_type_codes::ticket_type_id.eq_any(ticket_type_ids)),
            ).execute(conn),
        )
    }

    pub fn ticket_type_code_ticket_type_id_valid(
        code_id: Uuid,
        ticket_type_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Result<(), ValidationError>, DatabaseError> {
        let result = select(ticket_type_code_ticket_type_id_valid(
            code_id,
            ticket_type_id,
        )).get_result::<bool>(conn)
        .to_db_error(
            ErrorCode::InsertError,
            "Could not confirm if redemption code unique",
        )?;
        if !result {
            let mut validation_error = ValidationError::new("invalid");
            validation_error.add_param(Cow::from("ticket_type_id"), &ticket_type_id);

            return Ok(Err(validation_error));
        }
        Ok(Ok(()))
    }
}
