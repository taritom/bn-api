use chrono::NaiveDateTime;
use diesel::prelude::*;
use models::*;
use rand::prelude::*;
use uuid::Uuid;

use test::builders::event_builder::EventBuilder;

pub struct BroadcastBuilder<'a> {
    event_id: Option<Uuid>,
    notification_type: BroadcastType,
    channel: BroadcastChannel,
    name: String,
    message: Option<String>,
    send_at: Option<NaiveDateTime>,
    status: BroadcastStatus,
    connection: &'a PgConnection,
    subject: Option<String>,
    audience: BroadcastAudience,
}

impl<'a> BroadcastBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        let x: u32 = random();
        BroadcastBuilder {
            event_id: None,
            notification_type: BroadcastType::LastCall,
            channel: BroadcastChannel::PushNotification,
            name: format!("Broadcast {}", x).into(),
            message: None,
            send_at: None,
            status: BroadcastStatus::Pending,
            subject: None,
            audience: BroadcastAudience::PeopleAtTheEvent,
            connection,
        }
    }

    pub fn with_subject(mut self, subject: Option<String>) -> Self {
        self.subject = subject;
        self
    }

    pub fn with_audience(mut self, audience: BroadcastAudience) -> Self {
        self.audience = audience;
        self
    }

    pub fn with_channel(mut self, channel: BroadcastChannel) -> Self {
        self.channel = channel;
        self
    }

    pub fn with_status(mut self, status: BroadcastStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_send_at(mut self, send_at: NaiveDateTime) -> Self {
        self.send_at = Some(send_at);
        self
    }

    pub fn with_event_id(mut self, event_id: Uuid) -> Self {
        self.event_id = Some(event_id);
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn finish(&mut self) -> Broadcast {
        if self.event_id.is_none() {
            self.event_id = Some(EventBuilder::new(self.connection).finish().id);
        }

        let broadcast = Broadcast::create(
            self.event_id.unwrap(),
            self.notification_type,
            self.channel,
            self.name.clone(),
            self.message.clone(),
            self.send_at,
            Some(self.status),
            None,
            BroadcastAudience::PeopleAtTheEvent,
            None,
        );

        broadcast.commit(self.connection).unwrap()
    }
}
