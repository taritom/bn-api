use diesel::prelude::*;
use models::*;
use rand::prelude::*;
use test::builders::*;
use uuid::Uuid;

pub struct HoldBuilder<'a> {
    name: String,
    redemption_code: String,
    event_id: Option<Uuid>,
    hold_type: HoldTypes,
    connection: &'a PgConnection,
}

impl<'a> HoldBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        let x: u16 = random();
        HoldBuilder {
            name: format!("Hold {}", x).into(),
            redemption_code: format!("REDEEM{}", x).into(),
            connection,
            hold_type: HoldTypes::Discount,
            event_id: None,
        }
    }

    pub fn with_hold_type(mut self, hold_type: HoldTypes) -> Self {
        self.hold_type = hold_type;
        self
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    pub fn with_event(mut self, event: &Event) -> Self {
        self.event_id = Some(event.id);
        self
    }

    pub fn finish(mut self) -> Hold {
        if self.event_id.is_none() {
            self.event_id = Some(
                EventBuilder::new(self.connection)
                    .with_ticket_pricing()
                    .with_tickets()
                    .finish()
                    .id,
            );
        }

        let event = Event::find(self.event_id.unwrap(), self.connection).unwrap();
        let ticket_type_id = event.ticket_types(self.connection).unwrap()[0].id;

        let hold = Hold::create(
            self.name,
            self.event_id.unwrap(),
            self.redemption_code,
            if self.hold_type == HoldTypes::Discount {
                Some(10)
            } else {
                None
            },
            None,
            None,
            self.hold_type,
            ticket_type_id,
        ).commit(self.connection)
        .unwrap();

        hold.set_quantity(10, self.connection).unwrap();
        hold
    }
}
