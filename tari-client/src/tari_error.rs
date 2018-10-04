use reqwest;
use serde_json;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
pub struct TariError {
    pub description: String,
    pub cause: Option<Arc<dyn Error>>,
}

impl Error for TariError {
    fn description(&self) -> &str {
        &self.description
    }
}

unsafe impl Send for TariError {}
unsafe impl Sync for TariError {}

impl fmt::Display for TariError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.description)
    }
}

impl From<reqwest::Error> for TariError {
    fn from(r: reqwest::Error) -> Self {
        TariError {
            description: format!("Error calling Tari: reqwest error {}", r),
            cause: Some(Arc::new(r)),
        }
    }
}

impl From<serde_json::Error> for TariError {
    fn from(r: serde_json::Error) -> Self {
        TariError {
            description: format!("Error deserializing response: {}", r),
            cause: Some(Arc::new(r)),
        }
    }
}
