extern crate r2d2_redis;
use r2d2_redis::{r2d2, redis, RedisConnectionManager};

use chrono;
use redis::{Client, Commands, Connection, RedisError, RedisResult};
use std::fmt::Error;
use std::string::ToString;

struct RedisServer {
    connection: Connection,
}

impl RedisServer {
    fn set_value(&mut self, key: &str, value: &str) -> Result<(), RedisError> {
        let _: () = self.connection.set(key.to_string(), value.to_string())?;
        Ok(())
    }

    fn get_value(&mut self, key: &str) -> redis::RedisResult<String> {
        self.connection.get(key.to_string())
    }
}
