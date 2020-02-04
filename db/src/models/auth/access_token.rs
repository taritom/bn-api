use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::{Uuid, ParseError};
use models::Scopes;
use itertools::Itertools;

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessToken {
    pub sub: String,
    pub iss: String,
    pub exp: u64,
    #[serde(default)]
    pub scopes: Option<Vec<String>>
}

impl AccessToken {
    pub fn new(user_id: &Uuid, issuer: String, expiry_in_minutes: &u64) -> Self {
        let mut timer = SystemTime::now();
        timer += Duration::from_secs(expiry_in_minutes * 60);
        let exp = timer.duration_since(UNIX_EPOCH).unwrap().as_secs();

        AccessToken {
            iss: issuer,
            sub: user_id.hyphenated().to_string(),
            exp,
            scopes: None
        }
    }

    pub fn new_limited_scope(user_id: &Uuid, issuer: String, expiry_in_minutes: &u64, scopes : Vec<Scopes>) -> Self {
        let mut timer = SystemTime::now();
        timer += Duration::from_secs(expiry_in_minutes * 60);
        let exp = timer.duration_since(UNIX_EPOCH).unwrap().as_secs();

        AccessToken {
            iss: issuer,
            sub: user_id.hyphenated().to_string(),
            exp,
            scopes: Some(scopes.into_iter().map(|s| s.to_string()).collect_vec())
        }
    }

    pub fn get_id(&self) -> Result<Uuid, ParseError> {
        Ok(Uuid::parse_str(&self.sub)?)
    }
}
