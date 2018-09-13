use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use models::{TicketPricingStatus, TicketType};
use schema::ticket_pricing;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable, PartialEq, Debug)]
#[belongs_to(TicketType)]
#[table_name = "ticket_pricing"]
pub struct TicketPricing {
    pub id: Uuid,
    ticket_type_id: Uuid,
    pub name: String,
    status: String,
    pub price_in_cents: i64,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

impl TicketPricing {
    pub fn create(ticket_type_id: Uuid, name: String, price_in_cents: i64) -> NewTicketPricing {
        NewTicketPricing {
            ticket_type_id,
            name,
            status: TicketPricingStatus::Published.to_string(),
            price_in_cents,
        }
    }

    pub fn status(&self) -> TicketPricingStatus {
        TicketPricingStatus::parse(&self.status).unwrap()
    }
}

#[derive(Insertable)]
#[table_name = "ticket_pricing"]
pub struct NewTicketPricing {
    ticket_type_id: Uuid,
    name: String,
    status: String,
    price_in_cents: i64,
}

impl NewTicketPricing {
    pub fn commit(self, conn: &PgConnection) -> Result<TicketPricing, DatabaseError> {
        diesel::insert_into(ticket_pricing::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not create ticket pricing")
    }
}
