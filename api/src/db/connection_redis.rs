use actix_web::{FromRequest, HttpRequest, Result};
use errors::BigNeonError;
use server::AppState;
use std::sync::Arc;
use r2d2_redis::RedisConnectionManager;
use r2d2_redis::redis::{RedisResult, Commands};
use chrono::{DateTime, Utc};
use std::borrow::Borrow;

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

pub trait RedisCommands {
    fn get_value(&mut self, key: &str) -> RedisResult<String>;
    fn set_value(&mut self, key: &str, value: &str) -> RedisResult<String>;
    fn get_value_int(&mut self, key: u32) -> RedisResult<u32>;
    fn set_value_int(&mut self, key: &str, value: u32) -> RedisResult<u32>;
    fn get_cache_value<T: ?Sized>(&mut self, query: &T, time_lapse: u32);
    fn set_cache_value<T: ?Sized>(&mut self, query: &T, cached_value: &str, time_lapse: u32);
    fn in_current_period(unix_time: u32, period: u32) -> bool;
}

impl RedisCommands for r2d2_redis::r2d2::PooledConnection<RedisConnectionManager> {
    fn get_value(&mut self, key: &str) -> RedisResult<String>{
        self.get(key)
    }
    fn set_value(&mut self, key: &str, value: &str) -> RedisResult<String> {
        self.set(key, value)
    }
    fn get_value_int(&mut self, key: &str) -> RedisResult<u32>{
        self.get(key)
    }
    fn set_value_int(&mut self, key: &str, value: u32) -> RedisResult<u32> {
        self.set(key, value)
    }

    // query: this is the structure that will be serialized to json
    // time_lapse: this is measured in seconds. Only return the redis value if it happened in this period
    fn get_cache_value<T: ?Sized>(&mut self, query: &T, time_lapse: u32) -> Option<String> {
        let query_serialized = serde_json::to_string(query)?;
        if let Some(set_time) = self.get_value_int(query_serialized.borrow()) {
            // get the time when query was set
            let b = self.in_current_period(set_time, time_lapse);
        }

        if let Some(seconds) = time_period {}
        format!("{}{}", foo, bar)
        match self.get_value(query_serialized.borrow()) {
            Ok(cached_value) => {
                Some(cached_value)
            },
            _ => None
        }
    }

    fn set_cache_value<T: ?Sized>(&mut self, query: &T, cached_value: &str, time_lapse: u32) {
        unimplemented!()
    }

    // returns true of the giving number of seconds falls in the current period
    // start_time: this is measured in Unix time, the time in milliseconds from 1970-01-01
    fn in_current_period(start_time: u32, period: u32) -> bool {
        let utc: DateTime<Utc> = Utc::now();
        if (utc.timestamp_millis() - start_time)*1000 > seconds {
            false
        }
        true
    }
}

impl ConnectionRedis {
    pub fn conn(self) -> Result<r2d2_redis::r2d2::PooledConnection<RedisConnectionManager>, r2d2::Error> {
        self.inner.get()
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
