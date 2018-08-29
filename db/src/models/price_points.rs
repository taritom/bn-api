use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{PricePointStatus, TicketType};
use schema::price_points;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable, PartialEq, Debug)]
#[belongs_to(TicketType)]
pub struct PricePoint {
    pub id: Uuid,
    ticket_type_id: Uuid,
    pub name: String,
    status: String,
    pub price_in_cents: i64,
    created_at: NaiveDateTime,
}

impl PricePoint {
    pub fn create(ticket_type_id: Uuid, name: String, price_in_cents: i64) -> NewPricePoint {
        NewPricePoint {
            ticket_type_id,
            name,
            status: PricePointStatus::Published.to_string(),
            price_in_cents,
        }
    }

    pub fn status(&self) -> PricePointStatus {
        PricePointStatus::parse(&self.status).unwrap()
    }
}

#[derive(Insertable)]
#[table_name = "price_points"]
pub struct NewPricePoint {
    ticket_type_id: Uuid,
    name: String,
    status: String,
    price_in_cents: i64,
}

impl NewPricePoint {
    pub fn commit(self, conn: &Connectable) -> Result<PricePoint, DatabaseError> {
        diesel::insert_into(price_points::table)
            .values(self)
            .get_result(conn.get_connection())
            .to_db_error(ErrorCode::InsertError, "Could not create price point")
    }
}
