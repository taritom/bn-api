use chrono::prelude::*;
use chrono::NaiveDate;
use chrono::NaiveDateTime;
use dev::builders::*;
use diesel::prelude::*;
use models::*;
use rand::prelude::*;
use time::Duration;
use uuid::Uuid;

pub struct EventBuilder<'a> {
    name: String,
    status: EventStatus,
    organization_id: Option<Uuid>,
    venue_id: Option<Uuid>,
    event_start: Option<NaiveDateTime>,
    connection: &'a PgConnection,
    with_tickets: bool,
    with_ticket_pricing: bool,
    fee_in_cents: Option<i64>,
}

impl<'a> EventBuilder<'a> {
    pub fn new(connection: &PgConnection) -> EventBuilder {
        let x: u16 = random();
        EventBuilder {
            name: format!("Event {}", x).into(),
            status: EventStatus::Published,
            organization_id: None,
            venue_id: None,
            event_start: None,
            connection,
            with_tickets: false,
            with_ticket_pricing: false,
            fee_in_cents: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_status(mut self, status: EventStatus) -> Self {
        self.status = status;
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

    pub fn with_ticket_pricing(mut self) -> Self {
        self.with_tickets = true;
        self.with_ticket_pricing = true;
        self
    }

    pub fn with_event_fee(mut self) -> Self {
        let x: i64 = random();
        self.fee_in_cents = Some(x % 500);
        self
    }

    pub fn finish(&mut self) -> Event {
        let organization_id = self
            .organization_id
            .or_else(|| Some(OrganizationBuilder::new(self.connection).finish().id))
            .unwrap();
        let event = Event::create(
            &self.name,
            self.status,
            organization_id,
            self.venue_id,
            self.event_start
                .or_else(|| Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))),
            Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(7, 8, 10)),
            None,
        ).commit(self.connection)
        .unwrap();

        let update_event_fee = EventEditableAttributes {
            fee_in_cents: self.fee_in_cents,
            ..Default::default()
        };

        let event = event.update(update_event_fee, self.connection).unwrap();

        if self.with_tickets {
            let early_bird_start = NaiveDateTime::from(Utc::now().naive_utc() - Duration::days(2));
            let early_bird_end = NaiveDateTime::from(Utc::now().naive_utc() - Duration::days(1));
            let standard_start = NaiveDateTime::from(Utc::now().naive_utc() - Duration::days(1));
            let standard_end = NaiveDateTime::from(Utc::now().naive_utc() + Duration::days(2));

            let event_start = NaiveDateTime::from(Utc::now().naive_utc() + Duration::days(2));
            let event_end = NaiveDateTime::from(Utc::now().naive_utc() + Duration::days(4));

            let wallet_id = event.issuer_wallet(self.connection).unwrap().id;

            let ticket_type = event
                .add_ticket_type(
                    "General Admission".to_string(),
                    100,
                    event_start,
                    event_end,
                    wallet_id,
                    self.connection,
                ).unwrap();

            if self.with_ticket_pricing {
                for t in event.ticket_types(self.connection).unwrap() {
                    t.add_ticket_pricing(
                        "Early bird".to_string(),
                        early_bird_start,
                        early_bird_end,
                        100,
                        self.connection,
                    ).unwrap();
                    t.add_ticket_pricing(
                        "Standard".to_string(),
                        standard_start,
                        standard_end,
                        150,
                        self.connection,
                    ).unwrap();
                }
            }

            let asset = Asset::find_by_ticket_type(&ticket_type.id, self.connection).unwrap();
            let _ = asset
                .update_blockchain_id(
                    format!("{}.{}", event.name, ticket_type.name).to_string(),
                    self.connection,
                ).unwrap();
        }

        event
    }
}
