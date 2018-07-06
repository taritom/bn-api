extern crate bigneon_api;

use bigneon_api::config::{Config, Environment};
use bigneon_api::server::Server;

fn main() {
    let config = Config::new(Environment::Development);
    Server::start(config);
}
