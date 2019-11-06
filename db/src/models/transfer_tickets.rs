use chrono::prelude::*;
use diesel;
use diesel::expression::dsl::count;
use diesel::prelude::*;
use models::*;
use schema::{transfer_tickets, transfers};
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;
use validator::*;
use validators::{self, *};

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "transfer_tickets"]
pub struct NewTransferTicket {
    pub ticket_instance_id: Uuid,
    pub transfer_id: Uuid,
}

#[derive(Queryable, Identifiable, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "transfer_tickets"]
pub struct TransferTicket {
    pub id: Uuid,
    pub ticket_instance_id: Uuid,
    pub transfer_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl TransferTicket {
    pub fn create(ticket_instance_id: Uuid, transfer_id: Uuid) -> NewTransferTicket {
        NewTransferTicket {
            ticket_instance_id,
            transfer_id,
        }
    }

    fn validate_no_pending_transfers(
        transfer_id: Option<Uuid>,
        ticket_instance_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Result<(), ValidationError>, DatabaseError> {
        let pending_transfers: i64 = transfers::table
            .inner_join(transfer_tickets::table.on(transfers::id.eq(transfer_tickets::transfer_id)))
            .filter(transfer_tickets::ticket_instance_id.eq(ticket_instance_id))
            .filter(transfers::status.eq(TransferStatus::Pending))
            .filter(transfers::id.ne(transfer_id.unwrap_or(Uuid::nil())))
            .select(count(transfers::id))
            .get_result(conn)
            .to_db_error(ErrorCode::QueryError, "Could not load validate pending transfers")?;

        if pending_transfers > 0 {
            return Ok(Err(create_validation_error(
                "too_many_pending_transfers",
                "An active pending transfer already exists for this ticket instance id",
            )));
        }

        Ok(Ok(()))
    }
}

impl NewTransferTicket {
    pub fn commit(&self, conn: &PgConnection) -> Result<TransferTicket, DatabaseError> {
        self.validate_record(conn)?;

        diesel::insert_into(transfer_tickets::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create new transfer ticket")
    }

    fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let mut validation_errors: Result<(), ValidationErrors> = Ok(());

        validation_errors = validators::append_validation_error(
            validation_errors,
            "ticket_instance_id",
            TransferTicket::validate_no_pending_transfers(None, self.ticket_instance_id, conn)?,
        );
        Ok(validation_errors?)
    }
}
