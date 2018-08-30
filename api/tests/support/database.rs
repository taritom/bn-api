use bigneon_api::config::{Config, Environment};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::db::Connectable;
use bigneon_db::dev::builders::*;
use diesel::prelude::*;
use diesel::Connection;
use std::sync::Arc;

pub struct TestDatabase {
    connection: TestConnection,
}

impl ConnectionGranting for TestDatabase {
    fn get_connection(&self) -> Box<Connectable> {
        Box::new(self.connection.clone())
    }
}

#[derive(Clone)]
pub struct TestConnection {
    connection: Arc<PgConnection>,
}

impl Connectable for TestConnection {
    fn get_connection(&self) -> &PgConnection {
        &self.connection
    }
}

impl TestDatabase {
    pub fn new() -> TestDatabase {
        let config = Config::new(Environment::Test);

        let connection = PgConnection::establish(&config.database_url).expect(&format!(
            "Connection to {} could not be established.",
            config.database_url
        ));

        connection.begin_test_transaction().unwrap();

        TestDatabase {
            connection: (TestConnection {
                connection: Arc::new(connection),
            }),
        }
    }

    pub fn create_artist(&self) -> ArtistBuilder {
        ArtistBuilder::new(&self.connection)
    }

    pub fn create_event(&self) -> EventBuilder {
        EventBuilder::new(&self.connection)
    }

    pub fn create_organization(&self) -> OrganizationBuilder {
        OrganizationBuilder::new(&self.connection)
    }

    pub fn create_organization_invite(&self) -> OrgInviteBuilder {
        OrgInviteBuilder::new(&self.connection)
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
}
