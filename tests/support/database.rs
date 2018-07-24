use bigneon_api::config::{Config, Environment};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::db::connections::Connectable;
use diesel::prelude::*;
use diesel::Connection;
use std::sync::Arc;

pub struct TestDatabase {
    connection: Arc<PgConnection>,
}

impl ConnectionGranting for TestDatabase {
    fn get_connection(&self) -> Box<Connectable> {
        Box::new(TestConnection {
            connection: self.connection.clone(),
        })
    }
}

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
            connection: Arc::new(connection),
        }
    }
}
