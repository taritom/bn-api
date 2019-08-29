use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::*;
use schema::event_users;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use utils::errors::*;
use uuid::Uuid;

#[derive(Identifiable, Queryable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "event_users"]
pub struct EventUser {
    pub id: Uuid,
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub role: Roles,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Serialize)]
#[table_name = "event_users"]
pub struct NewEventUser {
    pub user_id: Uuid,
    pub event_id: Uuid,
    pub role: Roles,
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "event_users"]
pub struct EventUserEditableAttributes {
    pub role: Option<Roles>,
}

impl NewEventUser {
    pub fn commit(&self, conn: &PgConnection) -> Result<EventUser, DatabaseError> {
        let result: EventUser = DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not add user to event",
            diesel::insert_into(event_users::table)
                .values(self)
                .get_result(conn),
        )?;

        Ok(result)
    }
}

impl EventUser {
    pub fn update_or_create(
        user_id: Uuid,
        event_ids: &[Uuid],
        role: Roles,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        for event_id in event_ids {
            if let Some(event_user) =
                EventUser::find_by_event_id_user_id(*event_id, user_id, conn).optional()?
            {
                event_user.update(&EventUserEditableAttributes { role: Some(role) }, conn)?;
            } else {
                EventUser::create(user_id, *event_id, role).commit(conn)?;
            }
        }
        Ok(())
    }

    pub fn find_by_event_id_user_id(
        event_id: Uuid,
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<EventUser, DatabaseError> {
        event_users::table
            .filter(event_users::event_id.eq(event_id))
            .filter(event_users::user_id.eq(user_id))
            .select(event_users::all_columns)
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Unable to load event user")
    }

    pub fn update(
        &self,
        attributes: &EventUserEditableAttributes,
        conn: &PgConnection,
    ) -> Result<EventUser, DatabaseError> {
        diesel::update(self)
            .set((attributes, event_users::updated_at.eq(dsl::now)))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update event user")
    }

    pub fn create(user_id: Uuid, event_id: Uuid, role: Roles) -> NewEventUser {
        NewEventUser {
            user_id,
            event_id,
            role,
        }
    }

    pub fn destroy(self, conn: &PgConnection) -> Result<(), DatabaseError> {
        diesel::delete(event_users::table.filter(event_users::id.eq(self.id)))
            .execute(conn)
            .to_db_error(ErrorCode::DeleteError, "Could not delete event user.")?;
        Ok(())
    }
}
