use diesel::prelude::*;
use models::*;
use rand::prelude::*;
use test::builders::*;
use uuid::Uuid;

#[allow(dead_code)]
pub struct EventReportSubscriberBuilder<'a> {
    event_id: Option<Uuid>,
    email: String,
    report_type: ReportTypes,
    connection: &'a PgConnection,
}

impl<'a> EventReportSubscriberBuilder<'a> {
    pub fn new(connection: &PgConnection) -> EventReportSubscriberBuilder {
        let x: u32 = random();
        EventReportSubscriberBuilder {
            event_id: None,
            email: format!("jeff{}@tari.com", x).into(),
            report_type: ReportTypes::TicketCounts,
            connection,
        }
    }

    pub fn with_event(mut self, event: &Event) -> EventReportSubscriberBuilder<'a> {
        self.event_id = Some(event.id);
        self
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }

    pub fn with_report_type(mut self, report_type: ReportTypes) -> Self {
        self.report_type = report_type;
        self
    }

    pub fn finish(&self) -> EventReportSubscriber {
        let event_id = self
            .event_id
            .or_else(|| Some(EventBuilder::new(self.connection).finish().id))
            .unwrap();

        EventReportSubscriber::create(event_id, self.report_type, self.email.clone())
            .commit(None, self.connection)
            .unwrap()
    }
}
