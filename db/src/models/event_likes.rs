use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{Event, User};
use schema::event_likes;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable, Serialize)]
#[belongs_to(User)]
#[belongs_to(Event)]
#[table_name = "event_likes"]
pub struct EventLike {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
}

#[derive(Insertable)]
#[table_name = "event_likes"]
pub struct NewEventLike {
    pub event_id: Uuid,
    pub user_id: Uuid,
}

impl NewEventLike {
    pub fn commit(&self, conn: &Connectable) -> Result<EventLike, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new event like",
            diesel::insert_into(event_likes::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl EventLike {
    pub fn create(event_id: Uuid, user_id: Uuid) -> NewEventLike {
        NewEventLike {
            event_id: event_id,
            user_id: user_id,
        }
    }
}
