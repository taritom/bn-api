extern crate bigneon_api;
extern crate dotenv;

use bigneon_api::config::{Config, Environment};
use bigneon_api::server::Server;
use dotenv::dotenv;

fn main() {
    dotenv().ok();
    let config = Config::new(Environment::Development);
    Server::start(config);
}
