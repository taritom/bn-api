use diesel::prelude::*;
use models::*;
use test::builders::*;
use uuid::Uuid;

pub struct RefundBuilder<'a> {
    user_id: Option<Uuid>,
    order_id: Option<Uuid>,
    connection: &'a PgConnection,
}

impl<'a> RefundBuilder<'a> {
    pub fn new(connection: &PgConnection) -> RefundBuilder {
        RefundBuilder {
            connection,
            user_id: None,
            order_id: None,
        }
    }

    pub fn for_user(mut self, user: &User) -> RefundBuilder<'a> {
        self.user_id = Some(user.id);
        self
    }

    pub fn with_order(mut self, order: &Order) -> RefundBuilder<'a> {
        self.order_id = Some(order.id);
        self
    }

    pub fn finish(mut self) -> Refund {
        if self.user_id.is_none() {
            let user = UserBuilder::new(self.connection).finish();
            self.user_id = Some(user.id);
        }

        if self.order_id.is_none() {
            let order = OrderBuilder::new(self.connection).finish();
            self.order_id = Some(order.id);
        }

        Refund::create(self.order_id.unwrap(), self.user_id.unwrap())
            .commit(self.connection)
            .unwrap()
    }
}
