use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{Event, User};
use schema::orders;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable)]
#[belongs_to(Event)]
#[belongs_to(User)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
}

#[derive(Insertable)]
#[table_name = "orders"]
pub struct NewOrder {
    pub user_id: Uuid,
    pub event_id: Uuid,
}

impl NewOrder {
    pub fn commit(&self, conn: &Connectable) -> Result<Order, DatabaseError> {
        use schema::orders;
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new order",
            diesel::insert_into(orders::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl Order {
    pub fn create(user_id: Uuid, event_id: Uuid) -> NewOrder {
        NewOrder {
            user_id: user_id,
            event_id: event_id,
        }
    }
}
