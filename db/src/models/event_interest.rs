use chrono::NaiveDateTime;
use db::Connectable;
use diesel;
use diesel::prelude::*;
use models::{Event, User};
use schema::event_interest;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use utils::errors::*;
use uuid::Uuid;

#[derive(Associations, Identifiable, Queryable, Serialize)]
#[belongs_to(User)]
#[belongs_to(Event)]
#[table_name = "event_interest"]
pub struct EventInterest {
    pub id: Uuid,
    pub event_id: Uuid,
    pub user_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable)]
#[table_name = "event_interest"]
pub struct NewEventInterest {
    pub event_id: Uuid,
    pub user_id: Uuid,
}

impl NewEventInterest {
    pub fn commit(&self, conn: &Connectable) -> Result<EventInterest, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new event like",
            diesel::insert_into(event_interest::table)
                .values(self)
                .get_result(conn.get_connection()),
        )
    }
}

impl EventInterest {
    pub fn create(event_id: Uuid, user_id: Uuid) -> NewEventInterest {
        NewEventInterest {
            event_id: event_id,
            user_id: user_id,
        }
    }

    pub fn remove(
        event_id: Uuid,
        user_id: Uuid,
        conn: &Connectable,
    ) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading organization",
            diesel::delete(
                event_interest::table
                    .filter(event_interest::user_id.eq(user_id))
                    .filter(event_interest::event_id.eq(event_id)),
            ).execute(conn.get_connection()),
        )
    }

    pub fn total_interest(event_id: Uuid, conn: &Connectable) -> Result<u32, DatabaseError> {
        let result = event_interest::table
            .filter(event_interest::event_id.eq(event_id))
            .load::<EventInterest>(conn.get_connection())
            .to_db_error(ErrorCode::QueryError, "Error loading event interest")?;

        Ok(result.len() as u32)
    }

    pub fn user_interest(
        event_id: Uuid,
        user_id: Uuid,
        conn: &Connectable,
    ) -> Result<bool, DatabaseError> {
        let result = event_interest::table
            .filter(event_interest::event_id.eq(event_id))
            .filter(event_interest::user_id.eq(user_id))
            .load::<EventInterest>(conn.get_connection())
            .to_db_error(ErrorCode::QueryError, "Error loading event interest")?;

        Ok(result.len() > 0)
    }
}
