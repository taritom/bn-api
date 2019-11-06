use diesel::prelude::*;
use prelude::*;
use test::builders::*;
use uuid::Uuid;

pub struct SettlementEntryBuilder<'a> {
    settlement_id: Option<Uuid>,
    event_id: Option<Uuid>,
    ticket_type_id: Option<Uuid>,
    face_value_in_cents: i64,
    revenue_share_value_in_cents: i64,
    online_sold_quantity: i64,
    fee_sold_quantity: i64,
    connection: &'a PgConnection,
}

impl<'a> SettlementEntryBuilder<'a> {
    pub fn new(connection: &PgConnection) -> SettlementEntryBuilder {
        SettlementEntryBuilder {
            settlement_id: None,
            event_id: None,
            ticket_type_id: None,
            face_value_in_cents: 100,
            revenue_share_value_in_cents: 10,
            online_sold_quantity: 2,
            fee_sold_quantity: 2,
            connection,
        }
    }

    pub fn with_face_value_in_cents(mut self, face_value_in_cents: i64) -> Self {
        self.face_value_in_cents = face_value_in_cents;
        self
    }

    pub fn with_revenue_share_value_in_cents(mut self, revenue_share_value_in_cents: i64) -> Self {
        self.revenue_share_value_in_cents = revenue_share_value_in_cents;
        self
    }

    pub fn with_online_sold_quantity(mut self, online_sold_quantity: i64) -> Self {
        self.online_sold_quantity = online_sold_quantity;
        self
    }

    pub fn with_event(mut self, event: &Event) -> Self {
        self.event_id = Some(event.id);
        self
    }

    pub fn with_settlement(mut self, settlement: &Settlement) -> Self {
        self.settlement_id = Some(settlement.id);
        self
    }

    pub fn with_ticket_type_id(mut self, ticket_type_id: Uuid) -> Self {
        self.ticket_type_id = Some(ticket_type_id);
        self
    }

    pub fn finish(&mut self) -> SettlementEntry {
        let event_id = self
            .event_id
            .or_else(|| Some(EventBuilder::new(self.connection).finish().id))
            .unwrap();
        let event = Event::find(event_id, self.connection).unwrap();
        let organization = Organization::find(event.organization_id, self.connection).unwrap();
        let settlement_id = self
            .settlement_id
            .or_else(|| {
                Some(
                    SettlementBuilder::new(self.connection)
                        .with_organization(&organization)
                        .finish()
                        .id,
                )
            })
            .unwrap();

        SettlementEntry::create(
            settlement_id,
            if self.ticket_type_id.is_some() {
                SettlementEntryTypes::TicketType
            } else {
                SettlementEntryTypes::EventFees
            },
            event_id,
            self.ticket_type_id,
            self.face_value_in_cents,
            self.revenue_share_value_in_cents,
            self.online_sold_quantity,
            self.fee_sold_quantity,
            self.online_sold_quantity * self.face_value_in_cents
                + self.fee_sold_quantity * self.revenue_share_value_in_cents,
        )
        .commit(self.connection)
        .unwrap()
    }
}
