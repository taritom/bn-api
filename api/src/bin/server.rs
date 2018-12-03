//#![deny(unused_extern_crates)]
extern crate bigneon_api;
extern crate dotenv;
#[macro_use]
extern crate log;
#[macro_use]
extern crate logging;
#[macro_use]
extern crate serde_json;
extern crate clap;

use bigneon_api::config::{Config, Environment};
use bigneon_api::server::Server;
use clap::*;
use dotenv::dotenv;
use log::Level::*;

fn main() {
    logging::setup_logger().unwrap();
    info!("Loading environment");
    dotenv().ok();
    jlog!(Info, "Environment loaded");

    let config = Config::new(Environment::Development);

    let matches = App::new("Big Neon API Server")
        .author("Big Neon")
        .version(crate_version!())
        .about("HTTP REST API server and event and scheduled tasks processor")
        .arg(
            Arg::with_name("process-actions")
                .help("Fetches and processes events and actions from the database")
                .short("t")
                .default_value("true"),
        ).arg(
            Arg::with_name("process-http")
                .help("Runs an HTTP API server processing requests")
                .short("a")
                .default_value("true"),
        ).arg(
            Arg::with_name("run-til-empty")
                .help("Process all pending domain actions and then exits")
                .short("b")
                .default_value("false"),
        ).get_matches();

    jlog!(Info, "Starting server", {"app_name": config.app_name});
    Server::start(
        config,
        matches
            .value_of("process-actions")
            .unwrap_or("true")
            .parse()
            .expect("Unknown value for `process-actions`. Use `true` or `false`"),
        matches
            .value_of("process-http")
            .unwrap_or("true")
            .parse()
            .expect("Unknown value for `process-http`. Use `true` or `false`"),
        matches
            .value_of("run-til-empty")
            .unwrap_or("false")
            .parse()
            .expect("Unknown value for `run-til-empty`. Use `true` or `false`"),
    );
    info!("Server shutting down");
}
