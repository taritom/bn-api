#![deny(unreachable_patterns)]
#![deny(unused_variables)]
#![deny(unused_imports)]
// Unused results is more often than not an error
#![deny(unused_must_use)]
extern crate reqwest;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

pub use self::charge_result::ChargeResult;
pub use self::customer::*;
pub use self::refund_result::RefundResult;
pub use self::stripe_client::StripeClient;
pub use self::stripe_error::StripeError;

mod charge_result;
mod customer;
mod refund_result;
mod stripe_client;
mod stripe_error;
