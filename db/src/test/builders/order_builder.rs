use diesel::prelude::*;
use models::*;
use test::builders::event_builder::EventBuilder;
use test::builders::user_builder::UserBuilder;
use uuid::Uuid;

pub struct OrderBuilder<'a> {
    user: Option<User>,
    ticket_type_id: Option<Uuid>,
    connection: &'a PgConnection,
    is_paid: bool,
}

impl<'a> OrderBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> OrderBuilder<'a> {
        OrderBuilder {
            connection,
            user: None,
            ticket_type_id: None,
            is_paid: false,
        }
    }

    pub fn for_user(mut self, user: &User) -> OrderBuilder<'a> {
        self.user = Some(user.clone());
        self
    }

    pub fn for_event(mut self, event: &Event) -> OrderBuilder<'a> {
        self.ticket_type_id = Some(event.ticket_types(&self.connection).unwrap()[0].id);
        self
    }

    pub fn is_paid(mut self) -> OrderBuilder<'a> {
        self.is_paid = true;
        self
    }

    pub fn finish(mut self) -> Order {
        if self.user.is_none() {
            let user = UserBuilder::new(self.connection).finish();
            self.user = Some(user);
        }
        if self.ticket_type_id.is_none() {
            let event = EventBuilder::new(self.connection)
                .with_ticket_pricing()
                .finish();
            self.ticket_type_id = Some(event.ticket_types(&self.connection).unwrap()[0].id);
        }

        let cart =
            Order::find_or_create_cart(self.user.as_ref().unwrap(), self.connection).unwrap();

        cart.add_tickets(self.ticket_type_id.unwrap(), 10, self.connection)
            .unwrap();
        let total = cart.calculate_total(self.connection).unwrap();

        let mut cart = cart;
        if self.is_paid {
            cart.add_external_payment(
                "blah".to_string(),
                self.user.unwrap().id,
                total,
                self.connection,
            ).unwrap();
        }

        cart
    }
}
