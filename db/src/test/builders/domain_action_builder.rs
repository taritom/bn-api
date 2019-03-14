use chrono::NaiveDateTime;
use diesel::prelude::*;
use models::*;
use uuid::Uuid;

pub struct DomainActionBuilder<'a> {
    domain_event_id: Option<Uuid>,
    domain_action_type: Option<DomainActionTypes>,
    communication_channel_type: Option<CommunicationChannelType>,
    payload: Option<serde_json::Value>,
    main_table: Option<String>,
    main_table_id: Option<Uuid>,
    scheduled_at: Option<NaiveDateTime>,
    expires_at: Option<NaiveDateTime>,
    status: Option<DomainActionStatus>,
    blocked_until: Option<NaiveDateTime>,
    max_attempt_count: Option<i64>,
    attempt_count: Option<i64>,

    connection: &'a PgConnection,
}

impl<'a> DomainActionBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        DomainActionBuilder {
            domain_event_id: None,
            domain_action_type: None,
            communication_channel_type: None,
            payload: None,
            main_table: None,
            main_table_id: None,
            scheduled_at: None,
            expires_at: None,
            status: None,
            blocked_until: None,
            attempt_count: None,
            max_attempt_count: None,

            connection,
        }
    }

    pub fn with_domain_event_id(mut self, domain_event_id: Uuid) -> Self {
        self.domain_event_id = Some(domain_event_id);
        self
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn with_main_table(mut self, main_table: String) -> Self {
        self.main_table = Some(main_table);
        self
    }

    pub fn with_main_table_id(mut self, main_table_id: Uuid) -> Self {
        self.main_table_id = Some(main_table_id);
        self
    }

    pub fn with_scheduled_at(mut self, scheduled_at: NaiveDateTime) -> Self {
        self.scheduled_at = Some(scheduled_at);
        self
    }

    pub fn with_blocked_until(mut self, blocked_until: NaiveDateTime) -> Self {
        self.blocked_until = Some(blocked_until);
        self
    }

    pub fn with_status(mut self, status: DomainActionStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_max_attempt_count(mut self, max_attempt_count: i64) -> Self {
        self.max_attempt_count = Some(max_attempt_count);
        self
    }

    pub fn with_attempt_count(mut self, attempt_count: i64) -> Self {
        self.attempt_count = Some(attempt_count);
        self
    }

    pub fn finish(self) -> DomainAction {
        let mut action = DomainAction::create(
            self.domain_event_id,
            self.domain_action_type
                .unwrap_or(DomainActionTypes::Communication),
            self.communication_channel_type,
            self.payload.unwrap_or(serde_json::Value::Null),
            self.main_table,
            self.main_table_id,
        );

        if self.scheduled_at.is_some() {
            action.schedule_at(self.scheduled_at.unwrap());
        }

        if self.blocked_until.is_some() {
            action.blocked_until = self.blocked_until.unwrap();
        }

        if self.expires_at.is_some() {
            action.expires_at = self.expires_at.unwrap();
        }

        if self.status.is_some() {
            action.status = self.status.unwrap();
        }

        if self.attempt_count.is_some() {
            action.attempt_count = self.attempt_count.unwrap();
        }

        if self.max_attempt_count.is_some() {
            action.max_attempt_count = self.max_attempt_count.unwrap();
        }

        action.commit(self.connection).unwrap()
    }
}
