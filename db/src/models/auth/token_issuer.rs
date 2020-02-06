use chrono::Duration;
use jsonwebtoken::errors::Error;
use models::Scopes;
use uuid::Uuid;

pub trait TokenIssuer {
    fn issue(&self, user_id: Uuid, expires: Duration) -> Result<String, Error>;
    fn issue_with_limited_scopes(&self, user_id: Uuid, scopes: Vec<Scopes>, expires: Duration)
        -> Result<String, Error>;
}
