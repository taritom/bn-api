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
    with_price_points: bool,
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
            with_price_points: false,
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

    pub fn with_price_points(mut self) -> Self {
        self.with_tickets = true;
        self.with_price_points = true;
        self
    }

    pub fn finish(&mut self) -> Event {
        let event = Event::create(
            &self.name,
            self.organization_id
                .or_else(|| Some(OrganizationBuilder::new(self.connection).finish().id))
                .unwrap(),
            self.venue_id,
            self.event_start
                .or_else(|| Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))),
            Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(7, 8, 10)),
            Some(NaiveDate::from_ymd(2016, 7, 1).and_hms(9, 10, 11)),
        ).commit(self.connection)
            .unwrap();

        if self.with_tickets {
            event
                .add_ticket_type("General Admission".to_string(), self.connection)
                .unwrap();

            if self.with_price_points {
                for t in event.ticket_types(self.connection).unwrap() {
                    t.add_price_point("Early bird".to_string(), 100, self.connection)
                        .unwrap();
                    t.add_price_point("Standard".to_string(), 200, self.connection)
                        .unwrap();
                }
            }
        }

        event
    }
}
