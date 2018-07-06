use bigneon_api::config::{Config, Environment};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::db::connections::Connectable;
use diesel::prelude::*;
use diesel::Connection;

pub struct TestDatabase {
    connection: PgConnection,
}

impl ConnectionGranting for TestDatabase {
    fn get_connection(&self) -> Box<Connectable> {
        Box::new(TestConnection {
            connection: self.connection,
        })
    }
}

pub struct TestConnection {
    connection: PgConnection,
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

        TestDatabase {
            connection: connection,
        }
    }
}
