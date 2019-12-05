use actix_web::{FromRequest, HttpRequest, Result};
use errors::BigNeonError;
use server::AppState;
use cache::RedisCacheConnection;

//impl RedisCacheConnection {
//    pub fn unwrap_body_to_string(response: &HttpResponse) -> Result<&str, &'static str> {
//        match response.body() {
//            Binary(binary) => Ok(str::from_utf8(binary.as_ref()).unwrap()),
//            _ => Err("Unexpected response body"),
//        }
//    }
//}

pub struct CacheDatabase{
    pub inner: Option<RedisCacheConnection>
}
impl Clone for CacheDatabase{
    fn clone(&self) -> Self {
        CacheDatabase{inner:self.inner.clone()}
    }
}

impl FromRequest<AppState> for CacheDatabase {
    type Config = ();
    type Result = Result<CacheDatabase, BigNeonError>;

    fn from_request(request: &HttpRequest<AppState>, config: &Self::Config) -> Self::Result {
        if let Some(connection) = request.extensions().get::<CacheDatabase>() {
            return Ok(connection.clone());
        }

        let connection = request.state().database.cache_database.clone();

        request.extensions_mut().insert(connection.clone());
        Ok(connection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use r2d2_redis::RedisConnectionManager;
    use errors::BigNeonError;

    fn create_redis_connection_pool(
        database_url: &str,
    ) -> Result<r2d2_redis::r2d2::Pool<RedisConnectionManager>, BigNeonError> {
        let manager = RedisConnectionManager::new(database_url)?;
        let pool = r2d2_redis::r2d2::Pool::builder().build(manager)?;
        Ok(pool)
    }
    #[test]
    fn test_caching() {
        let conn = create_redis_connection_pool("redis://127.0.0.1/").unwrap();
        assert_eq!(2 + 2, 4);
    }
}
