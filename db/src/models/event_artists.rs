use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{Artist, Event};
use schema::event_artists;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable)]
#[belongs_to(Event)]
#[belongs_to(Artist)]
#[table_name = "event_artists"]
pub struct EventArtist {
    pub id: Uuid,
    pub event_id: Uuid,
    pub artist_id: Uuid,
    pub rank: i32,
}

#[derive(Insertable)]
#[table_name = "event_artists"]
pub struct NewEventArtist {
    pub event_id: Uuid,
    pub artist_id: Uuid,
    pub rank: i32,
}

impl NewEventArtist {
    pub fn commit(&self, conn: &Connectable) -> Result<EventArtist, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not add artist to event",
            diesel::insert_into(event_artists::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl EventArtist {
    pub fn create(event_id: Uuid, artist_id: Uuid, rank: i32) -> NewEventArtist {
        NewEventArtist {
            event_id,
            artist_id,
            rank,
        }
    }
}
