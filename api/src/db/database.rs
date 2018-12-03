use config::Config;
use db::Connection;
use db::ConnectionType;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

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
        ConnectionType::R2D2(
            self.connection_pool
                .get()
                .expect("Failed to get connection from pool"),
        ).into()
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Database {
            connection_pool: self.connection_pool.clone(),
        }
    }
}

fn create_connection_pool(config: &Config) -> R2D2Pool {
    let r2d2_config = r2d2::Pool::builder()
        .max_size(config.database_pool_size)
        .min_idle(Some(1));

    let connection_manager = ConnectionManager::new(config.database_url.clone());

    r2d2_config
        .build(connection_manager)
        .expect("Failed to create connection pool.")
}
