use bigneon_db::db::Connectable;
use bigneon_db::db::DatabaseConnection;
use bigneon_db::dev::builders::*;
use diesel::dsl::sql;
use diesel::sql_types::Bool;
use diesel::{select, Connection, PgConnection, RunQueryDsl};
use dotenv::dotenv;
use std::env;

pub struct TestProject {
    pub connection: DatabaseConnection,
    admin: PgConnection,
}

#[allow(dead_code)]
impl TestProject {
    pub fn new() -> Self {
        dotenv().ok();
        let conn_str = env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be defined.");
        let admin_str =
            env::var("TEST_DATABASE_ADMIN_URL").expect("TEST_DATABASE_ADMIN_URL must be defined.");
        let connection =
            DatabaseConnection::new(&conn_str).expect("Could not connect to test database");
        let admin = PgConnection::establish(&admin_str)
            .expect("Could not get admin access to admin test database");
        connection
            .get_connection()
            .begin_test_transaction()
            .expect("Could not start testing transaction");
        TestProject { connection, admin }
    }

    pub fn db_exists(&self, name: &str) -> bool {
        select(sql::<Bool>(&format!(
            "EXISTS(SELECT 1 FROM pg_database WHERE datname='{}')",
            name
        ))).get_result(&self.admin)
            .unwrap()
    }

    pub fn table_exists(&self, table: &str) -> bool {
        select(sql::<Bool>(&format!(
            "EXISTS \
             (SELECT 1 \
             FROM information_schema.tables \
             WHERE table_name = '{}')",
            table
        ))).get_result(&self.admin)
            .unwrap()
    }

    pub fn create_user(&self) -> UserBuilder {
        UserBuilder::new(self)
    }

    pub fn create_organization(&self) -> OrganizationBuilder {
        OrganizationBuilder::new(self)
    }

    pub fn create_organization_invite(&self) -> OrgInviteBuilder {
        OrgInviteBuilder::new(self)
    }

    pub fn create_venue(&self) -> VenueBuilder {
        VenueBuilder::new(self)
    }

    pub fn create_event(&self) -> EventBuilder {
        EventBuilder::new(self)
    }

    pub fn create_artist(&self) -> ArtistBuilder {
        ArtistBuilder::new(self)
    }
}

/// Returns the database connection and starts a transaction that will never be committed
impl Connectable for TestProject {
    fn get_connection(&self) -> &PgConnection {
        &self.connection.get_connection()
    }
}
