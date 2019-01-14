use serde_json;
use std::error::Error;
use std::fmt;
use stripe::{StripeClient, StripeError};

pub trait PaymentProcessor: Sized {
    fn create_token_for_repeat_charges(
        &self,
        token: &str,
        description: &str,
    ) -> Result<RepeatChargeToken, PaymentProcessorError>;

    fn update_repeat_token(
        &self,
        repeat_token: &str,
        token: &str,
        description: &str,
    ) -> Result<RepeatChargeToken, PaymentProcessorError>;

    fn auth(
        &self,
        token: &str,
        amount: i64,
        currency: &str,
        description: &str,
        metadata: Vec<(String, String)>,
    ) -> Result<ChargeAuthResult, PaymentProcessorError>;

    fn refund(&self, auth_token: &str) -> Result<ChargeAuthResult, PaymentProcessorError>;

    fn partial_refund(
        &self,
        auth_token: &str,
        amount: u32,
    ) -> Result<ChargeAuthResult, PaymentProcessorError>;

    fn complete_authed_charge(
        &self,
        auth_token: &str,
    ) -> Result<ChargeResult, PaymentProcessorError>;
}

pub struct RepeatChargeToken {
    pub token: String,
    raw: String,
}

use serde_json::Error as SerdeError;
impl RepeatChargeToken {
    pub fn to_json(&self) -> Result<serde_json::Value, SerdeError> {
        serde_json::from_str(&self.raw)
    }
}

#[derive(Debug)]
pub struct PaymentProcessorError {
    pub description: String,

    pub cause: Option<Box<dyn Error>>,
}

unsafe impl Send for PaymentProcessorError {}
unsafe impl Sync for PaymentProcessorError {}

impl Error for PaymentProcessorError {
    fn description(&self) -> &str {
        &self.description
    }
}

impl fmt::Display for PaymentProcessorError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match &self.cause {
            Some(c) => write!(f, "{} caused by: {}", self.description, c.description()),
            None => write!(f, "{}", self.description),
        }
    }
}

pub struct ChargeAuthResult {
    pub id: String,
    raw: String,
}

impl ChargeAuthResult {
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.raw)
    }
}

#[derive(Debug)]
pub struct ChargeResult {
    pub id: String,
    raw: String,
}

impl ChargeResult {
    pub fn to_json(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::from_str(&self.raw)
    }
}

impl From<StripeError> for PaymentProcessorError {
    fn from(s: StripeError) -> PaymentProcessorError {
        PaymentProcessorError {
            description: s.description.clone(),
            cause: Some(Box::new(s)),
        }
    }
}

pub struct StripePaymentProcessor {
    client: StripeClient,
}

impl StripePaymentProcessor {
    pub fn new(stripe_secret_key: String) -> StripePaymentProcessor {
        StripePaymentProcessor {
            client: StripeClient::new(stripe_secret_key),
        }
    }
}

impl PaymentProcessor for StripePaymentProcessor {
    fn create_token_for_repeat_charges(
        &self,
        token: &str,
        description: &str,
    ) -> Result<RepeatChargeToken, PaymentProcessorError> {
        Ok(self
            .client
            .create_customer(description, token, Vec::<(String, String)>::new())
            .map(|r| RepeatChargeToken {
                token: r.id,
                raw: r.raw_data,
            })?)
    }

    fn update_repeat_token(
        &self,
        repeat_token: &str,
        token: &str,
        description: &str,
    ) -> Result<RepeatChargeToken, PaymentProcessorError> {
        Ok(self
            .client
            .update_customer(
                repeat_token,
                description,
                token,
                Vec::<(String, String)>::new(),
            )
            .map(|r| RepeatChargeToken {
                token: r.id,
                raw: r.raw_data,
            })?)
    }

    fn auth(
        &self,
        token: &str,
        amount: i64,
        currency: &str,
        description: &str,
        metadata: Vec<(String, String)>,
    ) -> Result<ChargeAuthResult, PaymentProcessorError> {
        Ok(self
            .client
            .auth(token, amount, currency, description, metadata)
            .map(|r| ChargeAuthResult {
                id: r.id,
                raw: r.raw_data,
            })?)
    }

    fn refund(&self, auth_token: &str) -> Result<ChargeAuthResult, PaymentProcessorError> {
        Ok(self.client.refund(auth_token).map(|r| ChargeAuthResult {
            id: r.id,
            raw: r.raw_data,
        })?)
    }

    fn partial_refund(
        &self,
        auth_token: &str,
        amount: u32,
    ) -> Result<ChargeAuthResult, PaymentProcessorError> {
        Ok(self
            .client
            .partial_refund(auth_token, amount)
            .map(|r| ChargeAuthResult {
                id: r.id,
                raw: r.raw_data,
            })?)
    }

    fn complete_authed_charge(
        &self,
        _auth_token: &str,
    ) -> Result<ChargeResult, PaymentProcessorError> {
        Ok(self.client.complete(_auth_token).map(|r| ChargeResult {
            id: r.id,
            raw: r.raw_data,
        })?)
    }
}
