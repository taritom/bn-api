extern crate r2d2_redis;
use r2d2_redis::{r2d2, redis, RedisConnectionManager};

use redis::{Connection, RedisResult, Commands, RedisError, Client};
use std::panic::resume_unwind;
use std::fmt::Error;
use std::string::ToString;

#[derive(Clone)]
pub struct RedisServer {
    pub connection_string: String,
    pub client: Client
}

impl RedisServer {
    fn create_connection(connection_string: &str) -> Result<RedisServer, RedisError> {
        let client = redis::Client::open(connection_string)?;
        let red = RedisServer {
            connection_string: connection_string.to_string(),
            client,
        };
        Ok(red)
    }

    fn set_value(self, key: &str, value: &str) -> Result<(), RedisError> {
        let mut connection = self.client.get_connection()?;
        let _: () = connection.set(key.to_string(), value.to_string())?;
        Ok(())
    }

    fn get_value(self, key: &str) -> redis::RedisResult<String> {
        let mut connection = self.client.get_connection()?;
        connection.get(key.to_string())
    }
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    #[test]
//    fn it_works() {
//        assert_eq!(2 + 2, 4);
//    }
//}
