use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{Artist, Event};
use schema::event_artists;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use utils::errors::*;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[belongs_to(Event)]
#[belongs_to(Artist)]
#[table_name = "event_artists"]
pub struct EventArtist {
    pub id: Uuid,
    pub event_id: Uuid,
    pub artist_id: Uuid,
    pub rank: i32,
    pub set_time: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "event_artists"]
pub struct NewEventArtist {
    pub event_id: Uuid,
    pub artist_id: Uuid,
    pub rank: i32,
    pub set_time: Option<NaiveDateTime>,
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
    pub fn create(
        event_id: Uuid,
        artist_id: Uuid,
        rank: i32,
        set_time: Option<NaiveDateTime>,
    ) -> NewEventArtist {
        NewEventArtist {
            event_id,
            artist_id,
            rank,
            set_time,
        }
    }

    pub fn find_all_from_event(
        event_id: Uuid,
        conn: &Connectable,
    ) -> Result<Vec<EventArtist>, DatabaseError> {
        let result = event_artists::table
            .filter(event_artists::event_id.eq(event_id))
            .load(conn.get_connection())
            .to_db_error(ErrorCode::QueryError, "Could not load event artist")?;

        Ok(result)
    }

    pub fn clear_all_from_event(event_id: Uuid, conn: &Connectable) -> Result<(), DatabaseError> {
        let _result = diesel::delete(
            event_artists::table.filter(event_artists::event_id.eq(event_id)),
        ).execute(conn.get_connection())
            .to_db_error(ErrorCode::DeleteError, "Could not delete event artists.")?;
        Ok(())
    }
}
