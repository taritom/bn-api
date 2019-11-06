use diesel::prelude::*;
use models::*;
use test::builders::UserBuilder;
use uuid::Uuid;

pub struct PaymentMethodBuilder<'a> {
    name: PaymentProviders,
    user_id: Option<Uuid>,
    is_default: bool,
    connection: &'a PgConnection,
}

impl<'a> PaymentMethodBuilder<'a> {
    pub fn new(connection: &'a PgConnection) -> Self {
        PaymentMethodBuilder {
            name: PaymentProviders::Stripe,
            user_id: None,
            is_default: false,
            connection,
        }
    }

    pub fn with_user(mut self, user: &User) -> PaymentMethodBuilder<'a> {
        self.user_id = Some(user.id);
        self
    }

    pub fn make_default(mut self) -> Self {
        self.is_default = true;
        self
    }

    pub fn with_name(mut self, name: PaymentProviders) -> Self {
        self.name = name;
        self
    }

    pub fn finish(mut self) -> PaymentMethod {
        if self.user_id.is_none() {
            let user = UserBuilder::new(self.connection).finish();
            self.user_id = Some(user.id);
        }

        let user_id = self.user_id.unwrap();

        PaymentMethod::create(user_id, self.name, self.is_default, "cus_example".into(), "abc".into())
            .commit(user_id, self.connection)
            .unwrap()
    }
}
