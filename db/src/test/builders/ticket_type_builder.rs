use chrono::NaiveDateTime;
use models::*;
use test::builders::event_builder::EventBuilder;
use utils::dates::IntoDateBuilder;

pub struct TicketTypeBuilder<'a> {
    event: EventBuilder<'a>,
    quantity: u32,
    sales_start: Option<NaiveDateTime>,
    sales_end: Option<NaiveDateTime>,
    visibility: TicketTypeVisibility,
}

impl<'a> TicketTypeBuilder<'a> {
    pub fn new(event: EventBuilder<'a>) -> TicketTypeBuilder<'a> {
        TicketTypeBuilder {
            event,
            quantity: 100,
            sales_start: None,
            sales_end: None,
            visibility: TicketTypeVisibility::Always,
        }
    }

    pub fn starting(mut self, date: NaiveDateTime) -> Self {
        self.sales_start = Some(date);
        self
    }

    pub fn ending(mut self, date: NaiveDateTime) -> Self {
        self.sales_end = Some(date);
        self
    }

    pub fn visibility(mut self, visibility: TicketTypeVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_pricing(self) -> TicketPricingBuilder<'a> {
        TicketPricingBuilder::new(self)
    }

    pub fn finish(mut self) -> Event {
        let connection = self.event.connection;
        let event = self.event.finish();
        let sales_start = event
            .event_start
            .unwrap()
            .into_builder()
            .add_days(-4)
            .finish();
        let sales_end = event
            .event_start
            .unwrap()
            .into_builder()
            .add_hours(-2)
            .finish();
        event
            .add_ticket_type(
                "Ticket Builder".to_string(),
                None,
                self.quantity,
                Some(self.sales_start.unwrap_or(sales_start)),
                self.sales_end.unwrap_or(sales_end),
                event.issuer_wallet(self.event.connection).unwrap().id,
                None,
                0,
                100,
                self.visibility,
                None,
                None,
                connection,
            )
            .unwrap();

        event
    }
}

pub struct TicketPricingBuilder<'a> {
    ticket_type: TicketTypeBuilder<'a>,
    start: Option<NaiveDateTime>,
    end: Option<NaiveDateTime>,
}

impl<'a> TicketPricingBuilder<'a> {
    pub fn new(ticket_type: TicketTypeBuilder<'a>) -> TicketPricingBuilder<'a> {
        TicketPricingBuilder {
            ticket_type,
            start: None,
            end: None,
        }
    }
    pub fn starting(mut self, date: NaiveDateTime) -> Self {
        self.start = Some(date);
        self
    }
    pub fn ending(mut self, date: NaiveDateTime) -> Self {
        self.end = Some(date);
        self
    }
    pub fn finish(self) -> Event {
        let connection = self.ticket_type.event.connection;
        let event = self.ticket_type.finish();

        let ticket_type = event
            .ticket_types(false, None, connection)
            .unwrap()
            .pop()
            .unwrap();
        ticket_type
            .add_ticket_pricing(
                "Ticket Pricing Builder".to_string(),
                self.start
                    .unwrap_or(ticket_type.start_date(connection).unwrap()),
                self.end.unwrap_or(ticket_type.end_date),
                10,
                false,
                None,
                None,
                connection,
            )
            .unwrap();
        event
    }
}
