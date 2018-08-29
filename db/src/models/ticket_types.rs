use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::PricePoint;
use models::{Event, TicketTypeStatus};
use schema::{price_points, ticket_types};
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Identifiable, Associations, Queryable, PartialEq, Debug)]
#[belongs_to(Event)]
pub struct TicketType {
    pub id: Uuid,
    pub event_id: Uuid,
    pub name: String,
    status: String,
    created_at: NaiveDateTime,
}

impl TicketType {
    pub fn create(event_id: Uuid, name: String) -> NewTicketType {
        NewTicketType {
            event_id,
            name,
            status: TicketTypeStatus::Published.to_string(),
        }
    }

    pub fn find_by_event_id(
        event_id: Uuid,
        conn: &Connectable,
    ) -> Result<Vec<TicketType>, DatabaseError> {
        ticket_types::table
            .filter(ticket_types::event_id.eq(event_id))
            .order_by(ticket_types::name)
            .load(conn.get_connection())
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load ticket types for event",
            )
    }

    pub fn price_points(&self, conn: &Connectable) -> Result<Vec<PricePoint>, DatabaseError> {
        price_points::table
            .filter(price_points::ticket_type_id.eq(self.id))
            .order_by(price_points::name)
            .load(conn.get_connection())
            .to_db_error(
                ErrorCode::QueryError,
                "Could not load price points for ticket type",
            )
    }

    pub fn add_price_point(
        &self,
        name: String,
        price_in_cents: i64,
        conn: &Connectable,
    ) -> Result<PricePoint, DatabaseError> {
        PricePoint::create(self.id, name, price_in_cents).commit(conn)
    }

    pub fn status(&self) -> TicketTypeStatus {
        TicketTypeStatus::parse(&self.status).unwrap()
    }
}

#[derive(Insertable)]
#[table_name = "ticket_types"]
pub struct NewTicketType {
    event_id: Uuid,
    name: String,
    status: String,
}

impl NewTicketType {
    pub fn commit(self, conn: &Connectable) -> Result<TicketType, DatabaseError> {
        diesel::insert_into(ticket_types::table)
            .values(self)
            .get_result(conn.get_connection())
            .to_db_error(ErrorCode::InsertError, "Could not create ticket type")
    }
}
