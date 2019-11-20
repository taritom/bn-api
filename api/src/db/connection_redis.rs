use actix_web::{FromRequest, HttpRequest, Result};
use db::ConnectionType;
use errors::BigNeonError;
use r2d2_redis::redis;
use server::AppState;
use std::sync::Arc;

pub struct ConnectionRedis {
    pub inner: redis::Connection,
}

impl ConnectionRedis {
    pub fn get(&self) -> &redis::Connection {
        &self.inner
    }
    pub fn new(conn: redis::Connection) -> ConnectionRedis{
        ConnectionRedis{inner: conn}
    }
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

        request.extensions_mut().insert(connection.clone());
        connection
    }
}
