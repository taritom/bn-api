extern crate bigneon_db;
extern crate diesel;
extern crate dotenv;
#[macro_use]
extern crate log;
extern crate log4rs;

use bigneon_db::db::{Connectable, DatabaseConnection};
use bigneon_db::models::User;
use bigneon_db::schema::users;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

const DATABASE_URL: &str = "DATABASE_URL";

fn main() {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    info!("Starting app");
    info!(target: "db", "Only in log file");

    // Load .env, but don't freak out if we can't
    dotenv().ok();

    let database_url =
        env::var(&DATABASE_URL).expect(&format!("{} must be defined.", DATABASE_URL));
    let connection = DatabaseConnection::new(&database_url).expect("Error connecting to DB");
    let results = users::table
        .filter(users::active.eq(true))
        .limit(5)
        .load::<User>(connection.get_connection())
        .expect("Error loading users");

    println!("Displaying {} users", results.len());
    for user in results {
        println!("{:10} {}", user.id, user.email);
    }
}
