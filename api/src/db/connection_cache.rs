use actix_web::{FromRequest, HttpRequest, Result};
use cache::RedisCacheConnection;
use errors::BigNeonError;
use server::AppState;

pub struct CacheDatabase {
    pub inner: Option<RedisCacheConnection>,
}
impl Clone for CacheDatabase {
    fn clone(&self) -> Self {
        CacheDatabase {
            inner: self.inner.clone(),
        }
    }
}

impl FromRequest<AppState> for CacheDatabase {
    type Config = ();
    type Result = Result<CacheDatabase, BigNeonError>;

    fn from_request(request: &HttpRequest<AppState>, _config: &Self::Config) -> Self::Result {
        if let Some(connection) = request.extensions().get::<CacheDatabase>() {
            return Ok(connection.clone());
        }

        let connection = request.state().database.cache_database.clone();

        request.extensions_mut().insert(connection.clone());
        Ok(connection)
    }
}

