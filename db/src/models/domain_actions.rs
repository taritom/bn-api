use chrono::{Duration, NaiveDateTime, Utc};
use diesel;
use diesel::expression::dsl;
use diesel::prelude::*;
use models::enums::*;
use schema::*;
use serde_json;
use utils::errors::*;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Identifiable, Queryable)]
pub struct DomainAction {
    pub id: Uuid,
    pub domain_event_id: Option<Uuid>,
    pub domain_action_type: String,
    pub communication_channel_type: Option<String>,
    pub payload: serde_json::Value,
    pub main_table: String,
    pub main_table_id: Uuid,
    pub scheduled_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub last_attempted_at: Option<NaiveDateTime>,
    pub attempt_count: i64,
    pub max_attempt_count: i64,
    pub status: String,
    pub last_failure_reason: Option<String>,
    pub blocked_until: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "domain_actions"]
pub struct DomainActionEditableAttributes {
    pub scheduled_at: Option<NaiveDateTime>,
    pub last_attempted_at: Option<NaiveDateTime>,
    pub attempt_count: Option<i64>,
    pub blocked_until: NaiveDateTime,
}

impl DomainAction {
    pub fn create(
        domain_event_id: Option<Uuid>,
        domain_action_type: DomainActionTypes,
        communication_channel_type: Option<CommunicationChannelType>,
        payload: serde_json::Value,
        main_table: String,
        main_table_id: Uuid,
        scheduled_at: NaiveDateTime,
        expires_at: NaiveDateTime,
        max_attempt_count: i64,
    ) -> NewDomainAction {
        NewDomainAction {
            domain_event_id,
            domain_action_type: domain_action_type.to_string(),
            communication_channel_type: communication_channel_type
                .map_or(None, |v| Some(v.to_string())),
            payload,
            main_table: main_table,
            main_table_id,
            scheduled_at,
            expires_at,
            last_attempted_at: None,
            attempt_count: 0,
            max_attempt_count,
            status: DomainActionStatus::Pending.to_string(),
        }
    }

    pub fn domain_action_type(&self) -> Result<DomainActionTypes, EnumParseError> {
        self.domain_action_type.parse::<DomainActionTypes>()
    }

    pub fn find_pending(
        domain_action_type: Option<DomainActionTypes>,
        conn: &PgConnection,
    ) -> Result<Vec<DomainAction>, DatabaseError> {
        let mut query = domain_actions::table
            .filter(domain_actions::scheduled_at.le(dsl::now))
            .filter(domain_actions::expires_at.gt(dsl::now))
            .filter(domain_actions::blocked_until.le(dsl::now))
            .filter(domain_actions::attempt_count.lt(domain_actions::max_attempt_count))
            .filter(domain_actions::status.eq(DomainActionStatus::Pending.to_string()))
            .into_boxed();

        if let Some(action_type) = domain_action_type {
            query = query.filter(domain_actions::domain_action_type.eq(action_type.to_string()));
        }

        query
            .load(conn)
            .to_db_error(ErrorCode::QueryError, "Error loading domain actions")
    }
    pub fn find(id: Uuid, conn: &PgConnection) -> Result<DomainAction, DatabaseError> {
        DatabaseError::wrap(
            ErrorCode::QueryError,
            "Error loading domain action",
            domain_actions::table.find(id).first::<DomainAction>(conn),
        )
    }

    pub fn set_busy(&self, timeout: i64, conn: &PgConnection) -> Result<bool, DatabaseError> {
        let timeout = Utc::now().naive_utc() + Duration::seconds(timeout);
        let db_blocked = DomainAction::find(self.id, conn)?;
        if db_blocked.blocked_until > Utc::now().naive_utc() {
            return Ok(false);
        };
        let result: Result<DomainAction, DatabaseError> = diesel::update(self)
            .filter(domain_actions::blocked_until.le(timeout))
            .set((
                domain_actions::blocked_until.eq(timeout),
                domain_actions::updated_at.eq(dsl::now),
            )).get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update Domain Action");
        if let Err(i) = result {
            return Err(i);
        };
        return Ok(true);
    }

    pub fn set_done(&self, conn: &PgConnection) -> Result<DomainAction, DatabaseError> {
        diesel::update(self)
            .set((
                domain_actions::status.eq(DomainActionStatus::Success.to_string()),
                domain_actions::updated_at.eq(dsl::now),
            )).get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update Domain Action")
    }

    /// Use this method if there was a transient failure in performing the action. In
    /// general, it is assumed that the action will succeed at a later stage. If the
    /// action should not be retried, use `errored` instead. If the number of retries
    /// is exceeded, the status will changed to `RetriedExceeded`.
    pub fn set_failed(
        &self,
        reason: &str,
        conn: &PgConnection,
    ) -> Result<DomainAction, DatabaseError> {
        if self.max_attempt_count <= self.attempt_count + 1 {
            diesel::update(self)
                .set((
                    domain_actions::last_failure_reason.eq(reason),
                    domain_actions::status.eq(DomainActionStatus::RetriesExceeded.to_string()),
                    domain_actions::attempt_count.eq(self.attempt_count + 1),
                    domain_actions::blocked_until.eq(dsl::now),
                    domain_actions::updated_at.eq(dsl::now),
                )).get_result(conn)
                .to_db_error(ErrorCode::UpdateError, "Could not update Domain Action")
        } else {
            // Intentionally leave checked out
            diesel::update(self)
                .set((
                    domain_actions::last_failure_reason.eq(reason),
                    domain_actions::attempt_count.eq(self.attempt_count + 1),
                    domain_actions::updated_at.eq(dsl::now),
                )).get_result(conn)
                .to_db_error(ErrorCode::UpdateError, "Could not update Domain Action")
        }
    }

    /// Call this method to indicate that the action has errored and should not be retried.
    /// If there is a chance that the action could succeed at a later stage, use `failed()`
    /// instead
    pub fn set_errored(
        &self,
        reason: &str,
        conn: &PgConnection,
    ) -> Result<DomainAction, DatabaseError> {
        diesel::update(self)
            .set((
                domain_actions::last_failure_reason.eq(reason),
                domain_actions::status.eq(DomainActionStatus::Errored.to_string()),
                domain_actions::blocked_until.eq(dsl::now),
                domain_actions::updated_at.eq(dsl::now),
            )).get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update Domain Action")
    }

    pub fn update(
        &self,
        attributes: &DomainActionEditableAttributes,
        conn: &PgConnection,
    ) -> Result<DomainAction, DatabaseError> {
        diesel::update(self)
            .set((attributes, domain_actions::updated_at.eq(dsl::now)))
            .get_result(conn)
            .to_db_error(ErrorCode::UpdateError, "Could not update Domain Action")
    }
}

#[derive(Insertable)]
#[table_name = "domain_actions"]
pub struct NewDomainAction {
    pub domain_event_id: Option<Uuid>,
    pub domain_action_type: String,
    pub communication_channel_type: Option<String>,
    pub payload: serde_json::Value,
    pub main_table: String,
    pub main_table_id: Uuid,
    pub scheduled_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub last_attempted_at: Option<NaiveDateTime>,
    pub attempt_count: i64,
    pub max_attempt_count: i64,
    pub status: String,
}

impl NewDomainAction {
    pub fn commit(self, conn: &PgConnection) -> Result<DomainAction, DatabaseError> {
        diesel::insert_into(domain_actions::table)
            .values(self)
            .get_result(conn)
            .to_db_error(ErrorCode::InsertError, "Could not insert domain message")
    }
}
