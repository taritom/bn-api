use config::Config;
use db::ConnectionType;
use db::{Connection, ReadonlyConnection};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use errors::BigNeonError;
use r2d2::Error as R2D2Error;
use r2d2_redis;
use r2d2_redis::RedisConnectionManager;

type R2D2Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    connection_pool: R2D2Pool,
    redis_connection_pool: r2d2_redis::r2d2::Pool<RedisConnectionManager>,
}

impl Database {
    pub fn from_config(config: &Config) -> Database {
        Database {
            connection_pool: create_connection_pool(&config, config.database_url.clone()),
            redis_connection_pool: create_redis_connection_pool(&config.redis_connection_string).unwrap(),
        }
    }

    pub fn readonly_from_config(config: &Config) -> Database {
        Database {
            connection_pool: create_connection_pool(&config, config.readonly_database_url.clone()),
            redis_connection_pool: create_redis_connection_pool(&config.redis_connection_string).unwrap(),
        }
    }

    pub fn get_connection(&self) -> Result<Connection, R2D2Error> {
        let conn = self.connection_pool.get()?;
        Ok(ConnectionType::R2D2(conn).into())
    }

    pub fn get_ro_connection(&self) -> Result<ReadonlyConnection, R2D2Error> {
        let conn = self.connection_pool.get()?;
        Ok(ConnectionType::R2D2(conn).into())
    }

    pub fn get_redis_connection(&self) -> r2d2_redis::r2d2::Pool<RedisConnectionManager> {
        self.redis_connection_pool.clone()
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            connection_pool: self.connection_pool.clone(),
            redis_connection_pool: self.redis_connection_pool.clone(),
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

fn create_redis_connection_pool(
    database_url: &str,
) -> Result<r2d2_redis::r2d2::Pool<RedisConnectionManager>, BigNeonError> {
    let manager = RedisConnectionManager::new(database_url)?;
    let pool = r2d2_redis::r2d2::Pool::builder().build(manager)?;
    Ok(pool)
}
