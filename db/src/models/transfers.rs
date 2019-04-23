use chrono::prelude::*;
use diesel;
use diesel::expression::dsl::{self, count};
use diesel::prelude::*;
use models::*;
use schema::transfers;
use serde_json::Value;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;
use validator::*;
use validators::{self, *};

#[derive(Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "transfers"]
pub struct NewTransfer {
    pub ticket_instance_id: Uuid,
    pub source_user_id: Uuid,
    pub transfer_expiry_date: NaiveDateTime,
    pub transfer_key: Uuid,
    pub status: TransferStatus,
}

#[derive(Queryable, Identifiable, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "transfers"]
pub struct Transfer {
    pub id: Uuid,
    pub ticket_instance_id: Uuid,
    pub source_user_id: Uuid,
    pub destination_user_id: Option<Uuid>,
    pub transfer_expiry_date: NaiveDateTime,
    pub transfer_key: Uuid,
    pub status: TransferStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Default, Deserialize)]
#[table_name = "transfers"]
pub struct TransferEditableAttributes {
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub status: Option<TransferStatus>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub destination_user_id: Option<Uuid>,
}

impl Transfer {
    pub fn find_active_pending_by_ticket_instance_ids(
        ticket_instance_ids: &[Uuid],
        conn: &PgConnection,
    ) -> Result<Vec<Transfer>, DatabaseError> {
        transfers::table
            .filter(transfers::ticket_instance_id.eq_any(ticket_instance_ids))
            .filter(transfers::status.eq(TransferStatus::Pending))
            .filter(transfers::transfer_expiry_date.gt(Utc::now().naive_utc()))
            .select(transfers::all_columns)
            .distinct()
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading transfers")
    }

    pub fn cancel(
        &self,
        user_id: Uuid,
        new_transfer_key: Option<Uuid>,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        if self.status != TransferStatus::Pending {
            return Err(DatabaseError::new(
                ErrorCode::UpdateError,
                Some("Transfer cannot be cancelled as it is no longer pending".to_string()),
            ));
        }

        let transfer = self.update(
            TransferEditableAttributes {
                status: Some(TransferStatus::Cancelled),
                ..Default::default()
            },
            conn,
        )?;

        for (id, table) in vec![
            (self.id, Tables::Transfers),
            (self.ticket_instance_id, Tables::TicketInstances),
        ] {
            DomainEvent::create(
                DomainEventTypes::TransferTicketCancelled,
                "Ticket transfer was cancelled".to_string(),
                table,
                Some(id),
                Some(user_id),
                Some(json!({"old_transfer_key": self.transfer_key, "new_transfer_key": &new_transfer_key })),
            )
            .commit(conn)?;
        }

        Ok(transfer)
    }

    pub fn complete(
        &self,
        destination_user_id: Uuid,
        additional_data: Option<Value>,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        if self.status != TransferStatus::Pending {
            return Err(DatabaseError::new(
                ErrorCode::UpdateError,
                Some("Transfer cannot be completed as it is no longer pending".to_string()),
            ));
        }

        let transfer = self.update(
            TransferEditableAttributes {
                status: Some(TransferStatus::Completed),
                destination_user_id: Some(destination_user_id),
                ..Default::default()
            },
            conn,
        )?;

        for (id, table) in vec![
            (self.id, Tables::Transfers),
            (self.ticket_instance_id, Tables::TicketInstances),
        ] {
            DomainEvent::create(
                DomainEventTypes::TransferTicketCompleted,
                "Transfer ticket completed".to_string(),
                table,
                Some(id),
                None,
                additional_data.clone(),
            )
            .commit(conn)?;
        }

        Ok(transfer)
    }

    fn validate_no_pending_transfers(
        id: Option<Uuid>,
        ticket_instance_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Result<(), ValidationError>, DatabaseError> {
        let pending_transfers: i64 = transfers::table
            .filter(transfers::ticket_instance_id.eq(ticket_instance_id))
            .filter(transfers::status.eq(TransferStatus::Pending))
            .filter(transfers::transfer_expiry_date.ge(Utc::now().naive_utc()))
            .filter(transfers::id.ne(id.unwrap_or(Uuid::nil())))
            .select(count(transfers::id))
            .get_result(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load validate pending transfers",
            )?;

        if pending_transfers > 0 {
            return Ok(Err(create_validation_error(
                "too_many_pending_transfers",
                "An active pending transfer already exists for this ticket instance id",
            )));
        }

        Ok(Ok(()))
    }

    fn validate_record(
        &self,
        update_attrs: &TransferEditableAttributes,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let mut validation_errors: Result<(), ValidationErrors> = Ok(());

        if update_attrs.status == Some(TransferStatus::Pending) {
            validation_errors = validators::append_validation_error(
                validation_errors,
                "ticket_instance_id",
                Transfer::validate_no_pending_transfers(
                    Some(self.id),
                    self.ticket_instance_id,
                    conn,
                )?,
            );
        }

        Ok(validation_errors?)
    }

    pub fn create(
        ticket_instance_id: Uuid,
        source_user_id: Uuid,
        transfer_key: Uuid,
        transfer_expiry_date: NaiveDateTime,
    ) -> NewTransfer {
        NewTransfer {
            ticket_instance_id,
            source_user_id,
            transfer_key,
            transfer_expiry_date,
            status: TransferStatus::Pending,
        }
    }

    pub fn update(
        &self,
        attributes: TransferEditableAttributes,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        self.validate_record(&attributes, conn)?;

        DatabaseError::wrap(
            ErrorCode::UpdateError,
            "Could not update transfer",
            diesel::update(self)
                .set((attributes, transfers::updated_at.eq(dsl::now)))
                .get_result(conn),
        )
    }
}

impl NewTransfer {
    pub fn commit(
        &self,
        additional_data: Option<Value>,
        conn: &PgConnection,
    ) -> Result<Transfer, DatabaseError> {
        self.validate_record(conn)?;
        let result: Transfer = DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new transfer",
            diesel::insert_into(transfers::table)
                .values(self)
                .get_result(conn),
        )?;

        for (id, table) in vec![
            (result.id, Tables::Transfers),
            (self.ticket_instance_id, Tables::TicketInstances),
        ] {
            DomainEvent::create(
                DomainEventTypes::TransferTicketStarted,
                "Transfer ticket started".to_string(),
                table,
                Some(id),
                Some(self.source_user_id),
                additional_data.clone(),
            )
            .commit(conn)?;
        }
        Ok(result)
    }

    fn validate_record(&self, conn: &PgConnection) -> Result<(), DatabaseError> {
        let mut validation_errors: Result<(), ValidationErrors> = Ok(());

        if self.status == TransferStatus::Pending {
            validation_errors = validators::append_validation_error(
                validation_errors,
                "ticket_instance_id",
                Transfer::validate_no_pending_transfers(None, self.ticket_instance_id, conn)?,
            );
        }
        Ok(validation_errors?)
    }
}
