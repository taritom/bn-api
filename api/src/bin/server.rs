//#![deny(unused_extern_crates)]
extern crate bigneon_api;
extern crate dotenv;
extern crate log;
extern crate log4rs;

use bigneon_api::config::{Config, Environment};
use bigneon_api::server::Server;
use dotenv::dotenv;

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    dotenv().ok();
    let config = Config::new(Environment::Development);
    Server::start(config);
}
