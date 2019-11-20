use config::Config;
use db::{Connection, ReadonlyConnection};
use db::{ConnectionRedis, ConnectionType};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use r2d2::Error as R2D2Error;
use r2d2_redis::redis::RedisError;
use r2d2_redis::{redis, RedisConnectionManager};
use std::ops::DerefMut;
use r2d2::Pool;
use r2d2::PooledConnection;

type R2D2Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    pub pg_connection_pool: R2D2Pool,
    pub redis_connection_pool: r2d2::Pool<RedisConnectionManager>,
}

impl Database {
    pub fn from_config(config: &Config) -> Database {
        Database {
            pg_connection_pool: create_connection_pool(&config, config.database_url.clone()),
            redis_connection_pool: create_connection_pool_redis(&config, &config.redis_connection_string),
        }
    }

    pub fn readonly_from_config(config: &Config) -> Database {
        Database {
            pg_connection_pool: create_connection_pool(&config, config.readonly_database_url.clone()),
            redis_connection_pool: create_connection_pool_redis(&config, &config.redis_connection_string), // TODO: Need to use READ ONLY connection string
        }
    }

    pub fn get_connection(&self) -> Result<Connection, R2D2Error> {
        let conn = self.pg_connection_pool.get()?;
        Ok(ConnectionType::R2D2(conn).into())
    }

    pub fn get_ro_connection(&self) -> Result<ReadonlyConnection, R2D2Error> {
        let conn = self.pg_connection_pool.get()?;
        Ok(ConnectionType::R2D2(conn).into())
    }

    pub fn get_redis_connection(&self) -> Result<r2d2::PooledConnection<RedisConnectionManager>, impl std::error::Error> {
        self.redis_connection_pool.get()
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            pg_connection_pool: self.pg_connection_pool.clone(),
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

fn create_connection_pool_redis(config: &Config, database_url: &str) -> r2d2::Pool<RedisConnectionManager> {
    let manager = RedisConnectionManager::new(database_url).unwrap();
    let pool = r2d2::Pool::builder()
        .build(manager)
        .unwrap();
    pool
}
