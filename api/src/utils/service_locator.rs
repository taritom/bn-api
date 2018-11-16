use config::Config;
use errors::*;
use payments::PaymentProcessor;
use payments::StripePaymentProcessor;

pub struct ServiceLocator {
    stripe_secret_key: String,
}

impl ServiceLocator {
    pub fn new(config: &Config) -> ServiceLocator {
        ServiceLocator {
            stripe_secret_key: config.stripe_secret_key.to_string(),
        }
    }

    pub fn create_payment_processor(
        &self,
        provider_name: &str,
    ) -> Result<impl PaymentProcessor, BigNeonError> {
        match provider_name {
            "stripe" => Ok(StripePaymentProcessor::new(
                self.stripe_secret_key.to_string(),
            )),
            _ => return Err(ApplicationError::new("Unknown payment provider".into()).into()),
        }
    }
}
