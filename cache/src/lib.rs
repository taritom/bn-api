use r2d2_redis::redis::{Commands};
use r2d2_redis::r2d2::{PooledConnection, Pool};
use r2d2_redis::{RedisConnectionManager};
use std::sync::Arc;

type Milliseconds = usize;

// Contract for the Cache
pub trait CacheConnection {
    fn create_connection_pool(database_url: &str) -> anyhow::Result<Self> where Self: Sized;
    fn get(&mut self, key: &str) -> anyhow::Result<String>;
    fn delete(&mut self, key: &str) -> anyhow::Result<()>;
    fn add(&mut self, key: &str, data: &str, ttl: Milliseconds) -> anyhow::Result<()>;
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
    fn get(&mut self, key: &str) -> anyhow::Result<String> {
        Ok(self.conn.get(key)?)
    }
    fn delete(&mut self, key: &str) -> anyhow::Result<()> {
        let _: () = self.conn.del(key.to_string())?;
        Ok(())
    }
    fn add(&mut self, key: &str, data: &str, ttl: Milliseconds) -> anyhow::Result<()> {
        let _: () = self.conn.set(key, data)?;
        // Set a key's time to live in milliseconds.
        let _: () = self.conn.pexpire(key, ttl)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    fn sleep(duration: Milliseconds){
        let duration = time::Duration::from_millis(duration as u64);
        thread::sleep(duration);
    }

    #[test]
    fn test_caching() {
        let mut conn = RedisCacheConnection::create_connection_pool("redis://127.0.0.1/").unwrap();
        // store key for 10 milliseconds
        conn.add("key", "value", 10).unwrap();
        assert_eq!("value", conn.get("key").unwrap());

        sleep(11); 
        // key should now be expired
        assert!(conn.get("key").is_err());
    }
}
