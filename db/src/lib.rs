// Quiet diesel warnings https://github.com/diesel-rs/diesel/issues/1785
#![allow(proc_macro_derive_resolution_fallback)]
// Force these as errors so that they are not lost in all the diesel warnings
#![deny(unreachable_patterns)]
#![deny(unknown_lints)]
#![deny(unused_variables)]
#![deny(unused_imports)]
// Unused results is more often than not an error
#![deny(unused_must_use)]
extern crate num;
#[macro_use]
extern crate diesel;
extern crate argon2rs;
extern crate chrono;
extern crate dotenv;
extern crate itertools;
extern crate log;
extern crate log4rs;
extern crate rand;
extern crate time;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate validator_derive;
extern crate tari_client;
extern crate validator;

pub mod models;
pub mod schema;
pub mod utils;
pub mod validators;

mod test;

pub mod dev {
    pub use test::*;
}
