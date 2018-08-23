use chrono::NaiveDate;
use db::Connectable;
use dev::builders::*;
use models::*;
use rand::prelude::*;
use uuid::Uuid;

pub struct EventBuilder<'a> {
    name: String,
    organization_id: Option<Uuid>,
    venue_id: Option<Uuid>,
    connection: &'a Connectable,
}

impl<'a> EventBuilder<'a> {
    pub fn new(connection: &Connectable) -> EventBuilder {
        let x: u16 = random();
        EventBuilder {
            name: format!("Event {}", x).into(),
            organization_id: None,
            venue_id: None,
            connection,
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

    pub fn finish(&mut self) -> Event {
        Event::create(
            &self.name,
            self.organization_id
                .or_else(|| Some(OrganizationBuilder::new(self.connection).finish().id))
                .unwrap(),
            self.venue_id
                .or_else(|| Some(VenueBuilder::new(self.connection).finish().id))
                .unwrap(),
            NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
        ).commit(self.connection)
            .unwrap()
    }
}
