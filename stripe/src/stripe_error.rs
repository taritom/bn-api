use reqwest;
use serde_json;
use std::error::Error;
use std::sync::Arc;

#[derive(Debug)]
pub struct StripeError {
    pub description: String,
    pub cause: Option<Arc<dyn Error>>,
}

impl Error for StripeError {
    fn description(&self) -> &str {
        &self.description
    }
}

unsafe impl Send for StripeError {}
unsafe impl Sync for StripeError {}

use std::fmt;

impl fmt::Display for StripeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match &self.cause {
            Some(c) => write!(f, "{} caused by: {}", self.description, c.description()),
            None => write!(f, "{}", self.description),
        }
    }
}

impl StripeError {
    pub fn from_response(response: &mut reqwest::Response) -> StripeError {
        StripeError {
            description: format!(
                "Error calling Stripe: HTTP Code {}: Body:{}",
                response.status(),
                response
                    .text()
                    .unwrap_or("<Error reading response body>".to_string())
            ),
            cause: None,
        }
    }
}

impl From<reqwest::Error> for StripeError {
    fn from(r: reqwest::Error) -> Self {
        StripeError {
            description: format!("Error calling Stripe: reqwest error {}", r),
            cause: Some(Arc::new(r)),
        }
    }
}

impl From<serde_json::Error> for StripeError {
    fn from(r: serde_json::Error) -> Self {
        StripeError {
            description: format!("Error deserializing response:{}", r),
            cause: Some(Arc::new(r)),
        }
    }
}
