use chrono::NaiveDate;
use chrono::NaiveDateTime;
use db::Connectable;
use dev::builders::*;
use models::*;
use rand::prelude::*;
use uuid::Uuid;

pub struct EventBuilder<'a> {
    name: String,
    organization_id: Option<Uuid>,
    venue_id: Option<Uuid>,
    event_start: Option<NaiveDateTime>,
    connection: &'a Connectable,
    with_tickets: bool,
}

impl<'a> EventBuilder<'a> {
    pub fn new(connection: &Connectable) -> EventBuilder {
        let x: u16 = random();
        EventBuilder {
            name: format!("Event {}", x).into(),
            organization_id: None,
            venue_id: None,
            event_start: None,
            connection,
            with_tickets: false,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_organization(mut self, organization: &Organization) -> Self {
        self.organization_id = Some(organization.id.clone());
        self
    }

    pub fn with_venue(mut self, venue: &Venue) -> Self {
        self.venue_id = Some(venue.id.clone());
        self
    }

    pub fn with_event_start(mut self, date: &NaiveDateTime) -> Self {
        self.event_start = Some(date.clone());
        self
    }

    pub fn with_tickets(mut self) -> Self {
        self.with_tickets = true;
        self
    }

    pub fn finish(&mut self) -> Event {
        let event = Event::create(
            &self.name,
            self.organization_id
                .or_else(|| Some(OrganizationBuilder::new(self.connection).finish().id))
                .unwrap(),
            self.venue_id
                .or_else(|| Some(VenueBuilder::new(self.connection).finish().id))
                .unwrap(),
            self.event_start
                .or_else(|| Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)))
                .unwrap(),
            NaiveDate::from_ymd(2016, 7, 8).and_hms(7, 8, 10),
            NaiveDate::from_ymd(2016, 7, 1).and_hms(9, 10, 11),
        ).commit(self.connection)
            .unwrap();

        if self.with_tickets {
            TicketAllocation::create(event.id, 100).commit(self.connection);
        }

        event
    }
}
