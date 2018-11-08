use bigneon_api::config::{Config, Environment};
use bigneon_db::dev::*;
use bigneon_db::prelude::*;
use diesel::Connection;
use diesel::PgConnection;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct TestDatabase {
    pub connection: Arc<PgConnection>,
}

#[allow(dead_code)]
impl TestDatabase {
    pub fn new() -> TestDatabase {
        let config = Config::new(Environment::Test);

        let connection = PgConnection::establish(&config.database_url).unwrap_or_else(|_| {
            panic!(
                "Connection to {} could not be established.",
                config.database_url
            )
        });

        connection.begin_test_transaction().unwrap();

        TestDatabase {
            connection: Arc::new(connection),
        }
    }

    pub fn create_organization_with_user(&self, user: &User, owner: bool) -> OrganizationBuilder {
        let organization_builder = self.create_organization();
        if owner {
            organization_builder.with_owner(&user)
        } else {
            organization_builder.with_user(&user)
        }
    }

    pub fn create_artist(&self) -> ArtistBuilder {
        ArtistBuilder::new(&self.connection)
    }

    pub fn create_cart(&self) -> OrderBuilder {
        OrderBuilder::new(&self.connection)
    }

    pub fn create_code(&self) -> CodeBuilder {
        CodeBuilder::new(&self.connection)
    }

    pub fn create_comp(&self) -> CompBuilder {
        CompBuilder::new(&self.connection)
    }

    pub fn create_event(&self) -> EventBuilder {
        EventBuilder::new(&self.connection)
    }

    pub fn create_hold(&self) -> HoldBuilder {
        HoldBuilder::new(&self.connection)
    }

    pub fn create_order(&self) -> OrderBuilder {
        OrderBuilder::new(&self.connection)
    }

    pub fn create_organization(&self) -> OrganizationBuilder {
        OrganizationBuilder::new(&self.connection)
    }

    pub fn create_organization_invite(&self) -> OrgInviteBuilder {
        OrgInviteBuilder::new(&self.connection)
    }

    pub fn create_payment_method(&self) -> PaymentMethodBuilder {
        PaymentMethodBuilder::new(&self.connection)
    }

    pub fn create_region(&self) -> RegionBuilder {
        RegionBuilder::new(&self.connection)
    }

    pub fn create_user(&self) -> UserBuilder {
        UserBuilder::new(&self.connection)
    }

    pub fn create_venue(&self) -> VenueBuilder {
        VenueBuilder::new(&self.connection)
    }

    pub fn create_fee_schedule(&self) -> FeeScheduleBuilder {
        FeeScheduleBuilder::new(&self.connection)
    }

    pub fn create_purchased_tickets(
        &self,
        user: &User,
        ticket_type_id: Uuid,
        quantity: u32,
    ) -> Vec<TicketInstance> {
        let mut cart = Order::find_or_create_cart(user, &self.connection).unwrap();
        cart.update_quantities(
            &[UpdateOrderItem {
                ticket_type_id,
                quantity,
                redemption_code: None,
            }],
            &self.connection,
        ).unwrap();

        let total = cart.calculate_total(&self.connection).unwrap();
        cart.add_external_payment("test".to_string(), user.id, total, &self.connection)
            .unwrap();
        TicketInstance::find_for_user(user.id, &self.connection).unwrap()
    }
}
