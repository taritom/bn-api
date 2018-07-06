use bigneon_db::db::connections::Connectable;
use config::Config;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use scheduled_thread_pool::ScheduledThreadPool;
use std::sync::Arc;

type R2D2Connection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;
type R2D2Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub trait ConnectionGranting {
    fn get_connection(&self) -> Box<Connectable>;
}

pub struct Database {
    connection_pool: R2D2Pool,
}

pub struct ConnectionWrapper {
    connection: R2D2Connection,
}

impl Connectable for ConnectionWrapper {
    fn get_connection(&self) -> &PgConnection {
        &*self.connection
    }
}

impl ConnectionGranting for Database {
    fn get_connection(&self) -> Box<Connectable> {
        Box::new(ConnectionWrapper {
            connection: self.connection_pool
                .get()
                .expect("Failed to get connection from pool"),
        })
    }
}

impl Database {
    pub fn from_config(config: &Config) -> Database {
        Database {
            connection_pool: create_connection_pool(&config),
        }
    }
}

fn create_connection_pool(config: &Config) -> R2D2Pool {
    let thread_pool = Arc::new(ScheduledThreadPool::new(3));
    let r2d2_config = r2d2::Pool::builder().max_size(10).thread_pool(thread_pool);

    let connection_manager = ConnectionManager::new(config.database_url.clone());

    r2d2_config
        .build(connection_manager)
        .expect("Failed to create connection pool.")
}
