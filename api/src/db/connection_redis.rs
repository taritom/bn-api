use actix_web::{FromRequest, HttpRequest, Result};
use db::*;
use diesel;
use diesel::connection::TransactionManager;
use diesel::Connection as DieselConnection;
use diesel::PgConnection;
use errors::BigNeonError;
use server::AppState;
use std::sync::Arc;
use r2d2_redis::RedisConnectionManager;

pub struct ConnectionRedis {
    pub inner: Arc<r2d2_redis::r2d2::Pool<RedisConnectionManager>>,
}

impl Clone for ConnectionRedis {
    fn clone(&self) -> Self {
        ConnectionRedis {
            inner: self.inner.clone(),
        }
    }
}

impl FromRequest<AppState> for ConnectionRedis {
    type Config = ();
    type Result = Result<ConnectionRedis, BigNeonError>;

    fn from_request(request: &HttpRequest<AppState>, _config: &Self::Config) -> Self::Result {
        if let Some(connection) = request.extensions().get::<ConnectionRedis>() {
            return Ok(connection.clone());
        }

        let connection = request.state().database.get_redis_connection();
        let connection = ConnectionRedis{inner: Arc::new(connection)};

        request.extensions_mut().insert(connection.clone());
        Ok(connection.clone())
    }
}
