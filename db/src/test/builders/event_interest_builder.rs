use diesel::prelude::*;
use models::{Event, EventInterest, User};
use test::builders::*;
use uuid::Uuid;

#[allow(dead_code)]
pub struct EventInterestBuilder<'a> {
    event_id: Option<Uuid>,
    user_id: Option<Uuid>,
    connection: &'a PgConnection,
}

impl<'a> EventInterestBuilder<'a> {
    pub fn new(connection: &PgConnection) -> EventInterestBuilder {
        EventInterestBuilder {
            event_id: None,
            user_id: None,
            connection,
        }
    }

    pub fn with_event(mut self, event: &Event) -> EventInterestBuilder<'a> {
        self.event_id = Some(event.id);
        self
    }

    pub fn with_user(mut self, user: &User) -> EventInterestBuilder<'a> {
        self.user_id = Some(user.id);
        self
    }

    pub fn finish(&self) -> EventInterest {
        let event_id = self
            .event_id
            .or_else(|| Some(EventBuilder::new(self.connection).finish().id))
            .unwrap();

        let user_id = self
            .user_id
            .or_else(|| Some(UserBuilder::new(self.connection).finish().id))
            .unwrap();

        EventInterest::create(event_id, user_id)
            .commit(self.connection)
            .unwrap()
    }
}
