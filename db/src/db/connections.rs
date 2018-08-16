use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use utils::errors::DatabaseError;
use utils::errors::ErrorCode;
pub trait Connectable {
    fn get_connection(&self) -> &PgConnection;
}

/// A wrapped Database connection, which includes some convenience functions for producing a connection.
/// It also negates the need for having libraries or applications that use this crate to require the
/// diesel crate(s) as well.
pub struct DatabaseConnection {
    connection: PgConnection,
    connection_string: String,
}

impl DatabaseConnection {
    /// Create a new connection manager. The values for the hostname, user passwords etc, are expected
    /// to be defined in environment variables. In particular
    /// * `PG_HOSTNAME` - The host address for the database
    /// * `PG_PORT` - The port to connect to (default: 5432)
    /// * `DATABASE_NAME` - The bigneon database name (default: bigneon)
    /// * `PG_PASSWORD` - The password for the database user
    /// * `PG_USER` - The password for the database user
    pub fn new_from_env() -> Result<DatabaseConnection, DatabaseError> {
        dotenv().ok();
        // Required envars
        let hostname = env::var("PG_HOSTNAME").map_err(|_| {
            DatabaseError::new(
                ErrorCode::MissingInput,
                Some("PG_HOSTNAME must be defined."),
            )
        })?;
        let user = env::var("PG_USER").map_err(|_| {
            DatabaseError::new(ErrorCode::MissingInput, Some("PG_USER must be defined."))
        })?;
        let password = env::var("PG_PASSWORD").map_err(|_| {
            DatabaseError::new(
                ErrorCode::MissingInput,
                Some("PG_PASSWORD must be defined."),
            )
        })?;
        // Optional envars
        let port = env::var("PG_PORT").unwrap_or("5432".into());
        let database = env::var("DATABASE_NAME").unwrap_or("bigneon".into());
        let connection_string =
            DatabaseConnection::build_url(&hostname, &port, &database, &user, &password);
        DatabaseConnection::new(&connection_string)
    }

    pub fn new(connection_string: &str) -> Result<DatabaseConnection, DatabaseError> {
        let connection = PgConnection::establish(&connection_string)?;
        let db = DatabaseConnection {
            connection,
            connection_string: connection_string.into(),
        };
        Ok(db)
    }

    pub fn build_url(host: &str, port: &str, db: &str, user: &str, pw: &str) -> String {
        format!("postgres://{}:{}@{}:{}/{}", user, pw, host, port, db)
    }

    pub fn url(&self) -> &str {
        &self.connection_string
    }
}

impl Connectable for DatabaseConnection {
    fn get_connection(&self) -> &PgConnection {
        &self.connection
    }
}

#[test]
fn test_build_url() {
    let url = DatabaseConnection::build_url("localhost", "50432", "mydb", "jim", "password123!");
    assert_eq!(url, "postgres://jim:password123!@localhost:50432/mydb");
}
