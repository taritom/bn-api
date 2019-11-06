use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use models::TicketInstance;
use schema::*;
use utils::errors::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable, Serialize, Deserialize)]
pub struct RefundedTicket {
    pub id: Uuid,
    pub order_item_id: Uuid,
    pub ticket_instance_id: Uuid,
    pub fee_refunded_at: Option<NaiveDateTime>,
    pub ticket_refunded_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl RefundedTicket {
    pub fn create(order_item_id: Uuid, ticket_instance_id: Uuid) -> NewRefundedTicket {
        NewRefundedTicket {
            order_item_id,
            ticket_instance_id,
        }
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<RefundedTicket, DatabaseError> {
        refunded_tickets::table
            .filter(refunded_tickets::id.eq(id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve refunded ticket data")
    }

    pub fn find_by_ticket_instance_ids(
        ticket_instance_ids: Vec<Uuid>,
        conn: &PgConnection,
    ) -> Result<Vec<RefundedTicket>, DatabaseError> {
        refunded_tickets::table
            .filter(refunded_tickets::ticket_instance_id.eq_any(ticket_instance_ids))
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Could not retrieve refunded ticket data")
    }

    pub fn find_or_create_by_ticket_instance(
        ticket_instance: &TicketInstance,
        conn: &PgConnection,
    ) -> Result<RefundedTicket, DatabaseError> {
        if let Some(order_item_id) = ticket_instance.order_item_id {
            let refunded_ticket = refunded_tickets::table
                .filter(refunded_tickets::ticket_instance_id.eq(ticket_instance.id))
                .filter(refunded_tickets::order_item_id.eq(order_item_id))
                .first(conn)
                .optional()
                .to_db_error(ErrorCode::QueryError, "Could not retrieve refunded ticket data")?;

            Ok(refunded_ticket.unwrap_or(RefundedTicket::create(order_item_id, ticket_instance.id).commit(conn)?))
        } else {
            return DatabaseError::business_process_error(
                "Ticket must have an associated order item id to be refunded",
            );
        }
    }

    pub fn mark_ticket_and_fee_refunded(&mut self, conn: &PgConnection) -> Result<(), DatabaseError> {
        self.mark_refunded(false, conn)
    }

    pub fn mark_fee_only_refunded(&mut self, conn: &PgConnection) -> Result<(), DatabaseError> {
        self.mark_refunded(true, conn)
    }

    pub fn mark_refunded(&mut self, just_fee: bool, conn: &PgConnection) -> Result<(), DatabaseError> {
        self.updated_at = Utc::now().naive_utc();

        if !just_fee && self.ticket_refunded_at.is_none() {
            self.ticket_refunded_at = Some(self.updated_at);
        }

        if self.fee_refunded_at.is_none() {
            self.fee_refunded_at = Some(self.updated_at);
        }

        diesel::update(refunded_tickets::table.filter(refunded_tickets::id.eq(self.id)))
            .set((
                refunded_tickets::updated_at.eq(self.updated_at),
                refunded_tickets::fee_refunded_at.eq(self.fee_refunded_at),
                refunded_tickets::ticket_refunded_at.eq(self.ticket_refunded_at),
            ))
            .execute(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not mark record as refunded")?;

        Ok(())
    }
}

#[derive(Insertable, Clone)]
#[table_name = "refunded_tickets"]
pub struct NewRefundedTicket {
    pub order_item_id: Uuid,
    pub ticket_instance_id: Uuid,
}

impl NewRefundedTicket {
    pub fn commit(self, conn: &PgConnection) -> Result<RefundedTicket, DatabaseError> {
        diesel::insert_into(refunded_tickets::table)
            .values(&self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not insert refunded ticket")
    }
}
