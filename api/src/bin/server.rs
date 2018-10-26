//#![deny(unused_extern_crates)]
extern crate bigneon_api;
extern crate dotenv;
#[macro_use]
extern crate log;
#[macro_use]
extern crate logging;
#[macro_use]
extern crate serde_json;

use bigneon_api::config::{Config, Environment};
use bigneon_api::server::Server;
use dotenv::dotenv;
use log::Level::*;
use logging::*;

fn main() {
    logging::setup_logger().unwrap();
    info!("Loading environment");
    dotenv().ok();
    jlog!(Info, "Environment loaded");
    let config = Config::new(Environment::Development);
    jlog!(Info, "Starting server", {"app_name": config.app_name});
    Server::start(config);
    info!("Server running");
}
