use config::Config;
use db::ConnectionType;
use db::{Connection, ReadonlyConnection};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use errors::{ApplicationError, BigNeonError};

type R2D2Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub enum DatabaseConnectionPool {
    PGConn(r2d2::Pool<ConnectionManager<PgConnection>>),
    Redis(r2d2::Pool<ConnectionManager<PgConnection>>),
}

pub struct Database {
    pub connection_pool: DatabaseConnectionPool,
}


impl Database {
    pub fn from_config(config: &Config) -> Database {
        Database {
            connection_pool: DatabaseConnectionPool::PGConn(create_connection_pool(&config, config.database_url.clone())),
        }
    }

    pub fn readonly_from_config(config: &Config) -> Database {
        Database {
            connection_pool: DatabaseConnectionPool::PGConn(create_connection_pool(&config, config.readonly_database_url.clone())),
        }
    }

    pub fn get_connection(&self) -> Result<Connection, BigNeonError> {
        if let DatabaseConnectionPool::PGConn(c) = &self.connection_pool{
            let conn = c.get()?;
            return Ok(ConnectionType::R2D2(conn).into())
        }
        Err(BigNeonError::from(ApplicationError::new("Database connection pool does not exist".to_string())))
    }

    pub fn get_ro_connection(&self) -> Result<ReadonlyConnection, BigNeonError> {
        if let DatabaseConnectionPool::PGConn(c) = &self.connection_pool{
            let conn = c.get()?;
            return Ok(ConnectionType::R2D2(conn).into())
        }
        Err(BigNeonError::from(ApplicationError::new("Database connection pool does not exist".to_string())))
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            connection_pool: self.connection_pool.clone(),
        }
    }
}

fn create_connection_pool(config: &Config, database_url: String) -> R2D2Pool {
    let r2d2_config = r2d2::Pool::builder()
        .min_idle(Some(config.connection_pool.min))
        .max_size(config.connection_pool.max);

    let connection_manager = ConnectionManager::new(database_url);

    r2d2_config
        .build(connection_manager)
        .expect("Failed to create connection pool.")
}
