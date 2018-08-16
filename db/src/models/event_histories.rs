use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{Event, Order, User};
use schema::event_histories;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable)]
#[belongs_to(Event)]
#[belongs_to(Order)]
#[belongs_to(User)]
#[table_name = "event_histories"]
pub struct EventHistory {
    pub id: Uuid,
    pub event_id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub protocol_reference_hash: String,
}

#[derive(Insertable)]
#[table_name = "event_histories"]
pub struct NewEventHistory {
    pub event_id: Uuid,
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub protocol_reference_hash: String,
}

impl NewEventHistory {
    pub fn commit(&self, conn: &Connectable) -> Result<EventHistory, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new event history",
            diesel::insert_into(event_histories::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl EventHistory {
    pub fn create(
        event_id: Uuid,
        order_id: Uuid,
        user_id: Uuid,
        protocol_reference_hash: &str,
    ) -> NewEventHistory {
        NewEventHistory {
            event_id: event_id,
            order_id: order_id,
            user_id: user_id,
            protocol_reference_hash: String::from(protocol_reference_hash),
        }
    }
}
