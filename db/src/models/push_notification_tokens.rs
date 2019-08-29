use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use models::*;
use schema::push_notification_tokens;
use utils::errors::ConvertToDatabaseError;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
use uuid::Uuid;

#[derive(Default, Insertable, Serialize, Deserialize, PartialEq, Debug)]
#[table_name = "push_notification_tokens"]
pub struct NewPushNotificationToken {
    pub user_id: Uuid,
    pub token_source: String,
    pub token: String,
}

impl NewPushNotificationToken {
    pub fn commit(
        &self,
        user_id: Uuid,
        connection: &PgConnection,
    ) -> Result<PushNotificationToken, DatabaseError> {
        let push_notification_token: PushNotificationToken = DatabaseError::wrap(
            ErrorCode::InsertError,
            "Could not create new push_notification_token",
            diesel::insert_into(push_notification_tokens::table)
                .values(self)
                .get_result(connection),
        )?;
        push_notification_token.log_domain_event(user_id, connection)?;

        Ok(push_notification_token)
    }
}

#[derive(Queryable, Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct PushNotificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_source: String,
    pub token: String,
    pub last_notification_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl PushNotificationToken {
    pub fn log_domain_event(
        &self,
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<(), DatabaseError> {
        let mut domain_event = DomainEvent::create(
            DomainEventTypes::PushNotificationTokenCreated,
            "Push notification created".to_string(),
            Tables::PushNotificationTokens,
            Some(self.id),
            Some(user_id),
            None,
        );
        // Use created at date for push notification as domain event's created_at date
        domain_event.created_at = Some(self.created_at);
        domain_event.commit(conn)?;

        Ok(())
    }

    pub fn find(id: Uuid, conn: &PgConnection) -> Result<PushNotificationToken, DatabaseError> {
        push_notification_tokens::table
            .find(id)
            .first(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Error loading push notification token",
            )
    }

    pub fn create(user_id: Uuid, token_source: String, token: String) -> NewPushNotificationToken {
        NewPushNotificationToken {
            user_id,
            token_source,
            token,
        }
    }

    pub fn find_by_user_id(
        user_id: Uuid,
        conn: &PgConnection,
    ) -> Result<Vec<PushNotificationToken>, DatabaseError> {
        push_notification_tokens::table
            .filter(push_notification_tokens::user_id.eq(user_id))
            .select(push_notification_tokens::all_columns)
            .order_by(push_notification_tokens::token_source.asc())
            .then_order_by(push_notification_tokens::token.asc())
            .load(conn)
            .to_db_error(
                ErrorCode::QueryError,
                "Unable to load push_notification_tokens by user_id",
            )
    }

    pub fn remove(
        user_id: Uuid,
        push_notification_tokens_id: Uuid,
        conn: &PgConnection,
    ) -> Result<usize, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading push_notification_tokens",
            diesel::delete(
                push_notification_tokens::table
                    .filter(push_notification_tokens::user_id.eq(user_id))
                    .filter(push_notification_tokens::id.eq(push_notification_tokens_id)),
            )
            .execute(conn),
        )
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct DisplayPushNotificationToken {
    pub id: Uuid,
    pub token_source: String,
    pub token: String,
    pub last_notification_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl From<PushNotificationToken> for DisplayPushNotificationToken {
    fn from(push_notification_token: PushNotificationToken) -> Self {
        DisplayPushNotificationToken {
            id: push_notification_token.id,
            token_source: push_notification_token.token_source,
            token: push_notification_token.token,
            last_notification_at: push_notification_token.last_notification_at,
            created_at: push_notification_token.created_at,
        }
    }
}
