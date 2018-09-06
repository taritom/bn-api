use config::Config;
use db::Connection;
use db::ConnectionType;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use scheduled_thread_pool::ScheduledThreadPool;
use std::sync::Arc;

type R2D2Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct Database {
    connection_pool: R2D2Pool,
}

impl Database {
    pub fn from_config(config: &Config) -> Database {
        Database {
            connection_pool: create_connection_pool(&config),
        }
    }

    pub fn get_connection(&self) -> Connection {
        ConnectionType::R2D2(Arc::new(
            self.connection_pool
                .get()
                .expect("Failed to get connection from pool"),
        )).into()
    }
}

fn create_connection_pool(config: &Config) -> R2D2Pool {
    let thread_pool = Arc::new(ScheduledThreadPool::new(3));
    // TODO: This should be shared between threads
    let r2d2_config = r2d2::Pool::builder().max_size(2).thread_pool(thread_pool);

    let connection_manager = ConnectionManager::new(config.database_url.clone());

    r2d2_config
        .build(connection_manager)
        .expect("Failed to create connection pool.")
}
