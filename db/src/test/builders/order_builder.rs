use diesel::prelude::*;
use models::*;
use test::builders::event_builder::EventBuilder;
use test::builders::user_builder::UserBuilder;
use uuid::Uuid;

pub struct OrderBuilder<'a> {
    user_id: Option<Uuid>,
    ticket_type_id: Option<Uuid>,
    connection: &'a PgConnection,
}

impl<'a> OrderBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> OrderBuilder<'a> {
        OrderBuilder {
            connection,
            user_id: None,
            ticket_type_id: None,
        }
    }

    pub fn for_user(mut self, user: &User) -> OrderBuilder<'a> {
        self.user_id = Some(user.id);
        self
    }

    pub fn for_event(mut self, event: &Event) -> OrderBuilder<'a> {
        self.ticket_type_id = Some(event.ticket_types(&self.connection).unwrap()[0].id);
        self
    }

    pub fn finish(mut self) -> Order {
        if self.user_id.is_none() {
            let user = UserBuilder::new(self.connection).finish();
            self.user_id = Some(user.id);
        }
        if self.ticket_type_id.is_none() {
            let event = EventBuilder::new(self.connection)
                .with_ticket_pricing()
                .finish();
            self.ticket_type_id = Some(event.ticket_types(&self.connection).unwrap()[0].id);
        }
        let cart = Order::create(self.user_id.unwrap(), OrderTypes::Cart)
            .commit(self.connection)
            .unwrap();

        cart.add_tickets(self.ticket_type_id.unwrap(), 10, self.connection)
            .unwrap();

        cart
    }
}
