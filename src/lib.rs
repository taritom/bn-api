extern crate actix_web;
extern crate bigneon_db;
extern crate diesel;
extern crate dotenv;
extern crate scheduled_thread_pool;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate uuid;

pub mod config;
pub mod controllers;
pub mod database;
pub mod extractors;
pub mod server;

mod routing;
