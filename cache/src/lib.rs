use chrono::Utc;
use r2d2_redis::redis::{Commands};
use r2d2_redis::r2d2::{PooledConnection, Pool};
use r2d2_redis::{RedisConnectionManager};
use std::sync::Arc;

// Contract for the Cache
pub trait CacheConnection {
    fn create_connection_pool(database_url: &str) -> anyhow::Result<Self> where Self: Sized;
    fn get_value(&mut self, key: &str) -> anyhow::Result<String>;
    fn set_value(&mut self, key: &str, value: &str) -> anyhow::Result<String>;
    fn get_value_int(&mut self, key: &str) -> anyhow::Result<i64>;
    fn set_value_int(&mut self, key: &str, value: i64) -> anyhow::Result<i64>;
    fn delete(&mut self, key: &str);

    // start_time: this is measured in Unix time, the time in milliseconds from 1970-01-01
    // compares the difference in current time to giving
    fn is_key_outdated(&mut self, start_time: i64, seconds: i64) -> bool {
        Utc::now().timestamp_millis() - start_time > seconds
    }
    // time_lapse: this is measured in milli seconds. Only return the redis value if it happened in this period
    fn get_cache_value(&mut self, key: &str, time_lapse: i64) -> Option<String> {
        if let Some(set_time) = self.get_value_int(key).ok() {
            // get the time when query was set
            if !self.is_key_outdated(set_time, time_lapse) {
                // if not outdated return the value for the key
                // else return None
                return self.get_value(key).ok();
            }
        }
        None
    }

    fn set_cache_value(&mut self, key: &str, value: &str) {
        // set the current time and the new value for the key
        let time_now = Utc::now().timestamp_millis();
        self.set_value_int(key, time_now).ok();
        self.set_value(key, value).ok();
        ()
    }
}

//     pub fn unwrap_body_to_string(response: &HttpResponse) -> anyhow::Result<&str> {
//         match response.body() {
//             Binary(binary) => Ok(str::from_utf8(binary.as_ref()).unwrap()),
//             _ => Err("Unexpected response body"),
//         }
//     }
// }

// Implementation
pub struct RedisCacheConnection {
    pool: Arc<Pool<RedisConnectionManager>>,
    conn: PooledConnection<RedisConnectionManager>
}

impl Clone for RedisCacheConnection {
    fn clone(&self) -> Self {
        let pool = self.pool.clone();
        let conn = pool.get().unwrap();
        RedisCacheConnection { pool, conn}
    }
}

impl CacheConnection for RedisCacheConnection { // r2d2_redis::r2d2::PooledConnection<RedisConnectionManager> {
    fn create_connection_pool(
        database_url: &str,
    ) -> anyhow::Result<RedisCacheConnection> {
        let manager = RedisConnectionManager::new(database_url)?;
        let pool = r2d2_redis::r2d2::Pool::builder().build(manager)?;
        let conn = pool.get()?;
        Ok(RedisCacheConnection{ pool: Arc::from(pool), conn })
    }
    
    fn get_value(&mut self, key: &str) -> anyhow::Result<String> {
        Ok(self.conn.get(key)?)
    }
    fn set_value(&mut self, key: &str, value: &str) -> anyhow::Result<String> {
        Ok(self.conn.set(key, value)?)
    }
    fn get_value_int(&mut self, key: &str) -> anyhow::Result<i64> {
        Ok(self.conn.get(key)?)
    }
    fn set_value_int(&mut self, key: &str, value: i64) -> anyhow::Result<i64> {
        Ok(self.conn.set(key, value)?)
    }
    fn delete(&mut self, key: &str) {
        let _: () = self.conn.del(key.to_string()).unwrap_or_default();
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caching() {
        let mut conn = RedisCacheConnection::create_connection_pool("redis://127.0.0.1/").unwrap();
        conn.set_value("key", "value").unwrap();
        assert_eq!("value", conn.get_value("key").unwrap());

        print!("gggggl ");
        conn.set_value_int("key_int", 5).unwrap();
        print!("gccccggggl ");
        let t = conn.get_value_int("key_int");
        match t {
            Ok(t) => print!("t: {}", t),
            Err(t) => print!("error: {}", t),
        }
        // print!("jkjkjk {}", t);
        assert_eq!(5, conn.get_value_int("key_int").unwrap());
    }
}
