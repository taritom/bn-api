use diesel::prelude::*;
use models::*;
use test::builders::*;
use uuid::Uuid;

pub struct OrderBuilder<'a> {
    user: Option<User>,
    ticket_type_id: Option<Uuid>,
    connection: &'a PgConnection,
    quantity: u32,
    is_paid: bool,
    with_free_items: bool,
    on_behalf_of_user: Option<User>,
    external_payment_type: Option<ExternalPaymentType>,
    redemption_code: Option<String>,
    is_box_office: bool,
}

impl<'a> OrderBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> OrderBuilder<'a> {
        OrderBuilder {
            connection,
            user: None,
            ticket_type_id: None,
            quantity: 10,
            is_paid: false,
            with_free_items: false,
            on_behalf_of_user: None,
            external_payment_type: None,
            redemption_code: None,
            is_box_office: false,
        }
    }

    pub fn for_user(mut self, user: &User) -> OrderBuilder<'a> {
        self.user = Some(user.clone());
        self
    }

    pub fn box_office_order(mut self) -> OrderBuilder<'a> {
        self.is_box_office = true;
        self
    }

    pub fn on_behalf_of_user(mut self, user: &User) -> OrderBuilder<'a> {
        self.on_behalf_of_user = Some(user.clone());
        self.box_office_order()
    }

    pub fn for_event(mut self, event: &Event) -> OrderBuilder<'a> {
        self.ticket_type_id = Some(event.ticket_types(true, None, &self.connection).unwrap()[0].id);
        self
    }

    pub fn quantity(mut self, quantity: u32) -> OrderBuilder<'a> {
        self.quantity = quantity;
        self
    }

    pub fn is_paid(mut self) -> OrderBuilder<'a> {
        self.is_paid = true;
        self
    }

    pub fn with_redemption_code(mut self, redemption_code: String) -> OrderBuilder<'a> {
        self.redemption_code = Some(redemption_code);
        self
    }

    pub fn with_free_items(mut self) -> OrderBuilder<'a> {
        self.with_free_items = true;
        self
    }

    pub fn with_external_payment_type(
        mut self,
        external_payment_type: ExternalPaymentType,
    ) -> OrderBuilder<'a> {
        self.external_payment_type = Some(external_payment_type);
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
            self.ticket_type_id =
                Some(event.ticket_types(true, None, &self.connection).unwrap()[0].id);
        }

        let mut cart =
            Order::find_or_create_cart(self.user.as_ref().unwrap(), self.connection).unwrap();

        if self.with_free_items {
            let comp = HoldBuilder::new(self.connection)
                .with_ticket_type_id(self.ticket_type_id.unwrap())
                .with_hold_type(HoldTypes::Comp)
                .finish();
            self.redemption_code = comp.redemption_code;
        }

        let user = self.user.unwrap();

        cart.update_quantities(
            user.id,
            &[UpdateOrderItem {
                ticket_type_id: self.ticket_type_id.unwrap(),
                quantity: self.quantity,
                redemption_code: self.redemption_code,
            }],
            self.on_behalf_of_user.is_some(),
            self.is_box_office,
            self.connection,
        )
        .unwrap();

        if let Some(on_behalf_of_user) = self.on_behalf_of_user {
            cart.set_behalf_of_user(on_behalf_of_user, user.id, self.connection)
                .unwrap();
        }

        let total = cart.calculate_total(self.connection).unwrap();

        let mut cart = cart;
        if self.is_paid {
            if total == 0 {
                cart.add_free_payment(self.is_box_office, user.id, self.connection)
                    .unwrap();
            } else if self.is_box_office {
                cart.add_external_payment(
                    Some("blah".to_string()),
                    self.external_payment_type
                        .unwrap_or(ExternalPaymentType::CreditCard),
                    user.id,
                    total,
                    self.connection,
                )
                .unwrap();
            } else {
                cart.add_credit_card_payment(
                    user.id,
                    total,
                    PaymentProviders::Stripe,
                    "blah".to_string(),
                    PaymentStatus::Completed,
                    json!(""),
                    self.connection,
                )
                .unwrap();
            }
        }

        cart
    }
}
