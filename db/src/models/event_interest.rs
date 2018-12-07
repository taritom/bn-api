use chrono::NaiveDateTime;
use diesel;
use diesel::expression::dsl::count;
use diesel::prelude::*;
use models::*;
use schema::{event_interest, users};
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

#[derive(Serialize)]
pub struct DisplayEventInterestedUser {
    pub user_id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub thumb_profile_pic_url: Option<String>,
}

impl NewEventInterest {
    pub fn commit(&self, conn: &PgConnection) -> Result<EventInterest, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new event like",
            diesel::insert_into(event_interest::table)
                .values(self)
                .get_result(conn),
        )
    }
}

impl EventInterest {
    pub fn create(event_id: Uuid, user_id: Uuid) -> NewEventInterest {
        NewEventInterest { event_id, user_id }
    }

    pub fn remove(
        event_id: Uuid,
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading organization",
            diesel::delete(
                event_interest::table
                    .filter(event_interest::user_id.eq(user_id))
                    .filter(event_interest::event_id.eq(event_id)),
            )
            .execute(conn),
        )
    }

    pub fn total_interest(event_id: Uuid, conn: &PgConnection) -> Result<u32, DatabaseError> {
        let result = event_interest::table
            .filter(event_interest::event_id.eq(event_id))
            .load::<EventInterest>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading event interest")?;

        Ok(result.len() as u32)
    }

    pub fn user_interest(
        event_id: Uuid,
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<bool, DatabaseError> {
        let result = event_interest::table
            .filter(event_interest::event_id.eq(event_id))
            .filter(event_interest::user_id.eq(user_id))
            .load::<EventInterest>(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading event interest")?;

        Ok(!result.is_empty())
    }

    pub fn list_interested_users(
        event_id: Uuid,
        user_id: Uuid,
        page: u32,
        limit: u32,
        conn: &PgConnection,
    ) -> Result<Payload<DisplayEventInterestedUser>, DatabaseError> {
        //Request the total count of users with an interest for a specific event
        let total_interests: i64 = event_interest::table
            .filter(event_interest::event_id.eq(event_id))
            .filter(event_interest::user_id.ne(user_id)) //Remove primary user from results
            .select(count(event_interest::id))
            .first(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading event interest")?;
        if total_interests > 0 {
            //Request a pageable list of users with an interest for a specific event
            let event_interest_list = event_interest::table
                .filter(event_interest::event_id.eq(event_id))
                .filter(event_interest::user_id.ne(user_id)) //Remove primary user from results
                .inner_join(users::table)
                .select(users::all_columns)
                .order_by(event_interest::user_id.asc())
                .limit(limit as i64)
                .offset(limit as i64 * page as i64)
                .load::<User>(conn)
                .to_db_error(ErrorCode::QueryError, "Error loading event interest")?;
            //Keep only required user information
            let mut users: Vec<DisplayEventInterestedUser> = Vec::new();
            users.reserve(event_interest_list.len());
            for curr_user in &event_interest_list {
                let curr_entry = DisplayEventInterestedUser {
                    user_id: curr_user.id,
                    first_name: curr_user.clone().first_name.unwrap_or("".to_string()),
                    last_name: curr_user.clone().last_name.unwrap_or("".to_string()),
                    thumb_profile_pic_url: curr_user.thumb_profile_pic_url.clone(),
                };
                users.push(curr_entry);
            }
            let mut result = Payload::new(users, Paging::new(page, limit));
            result.paging.total = total_interests as u64;
            Ok(result)
        } else {
            let result = Payload::empty(Paging::new(page, limit));
            Ok(result)
        }
    }
}
