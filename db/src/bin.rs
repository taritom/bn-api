#[macro_use]
extern crate diesel_migrations;
extern crate clap;

#[macro_use]
extern crate diesel;
extern crate argon2rs;
extern crate chrono;
extern crate dotenv;
extern crate log;
extern crate rand;
extern crate uuid;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod db;
mod models;
mod schema;
mod utils;

embed_migrations!("./migrations");

use clap::ArgMatches;
use clap::{App, Arg, SubCommand};
use db::DatabaseConnection;
use diesel::pg::PgConnection;
use diesel::Connection;
use models::Roles;

pub fn main() {
    let matches = App::new("Big Neon DB CLI")
        .author("Big Neon")
        .about("Command Line Interface for creating and migrating the Big Neon database")
        .subcommand(
            SubCommand::with_name("migrate")
                .about("Migrates the database to the latest version")
                .arg(
                    Arg::with_name("connection")
                        .short("c")
                        .takes_value(true)
                        .help("Connection string to the database"),
                ),
        )
        .subcommand(
            SubCommand::with_name("create")
                .about("Creates a new instance of the database and inserts the system administrator user")
                .arg(
                    Arg::with_name("connection")
                        .short("c")
                        .takes_value(true)
                        .help("Connection string to the database"),
                )
                .arg(
                    Arg::with_name("email")
                        .short("e")
                        .takes_value(true)
                        .help("email for system administrator"),
                ).arg(
                Arg::with_name("phone")
                    .short("m")
                    .takes_value(true)
                    .help("phone number for system administrator"),
            ).arg(
                Arg::with_name("password")
                    .short("p")
                    .takes_value(true)
                    .help("password for system administrator"),
            ),
        )
        .get_matches();

    match matches.subcommand() {
        ("create", Some(matches)) => create_db_and_user(matches),
        ("migrate", Some(matches)) => migrate_db(matches),
        _ => unreachable!("The cli parser will prevent reaching here"),
    }
}

fn create_db(conn_string: &str) -> Result<(), diesel::result::Error> {
    let parts: Vec<&str> = conn_string.split("/").collect();
    let db = parts.last().unwrap();
    let db = str::replace(db, "'", "''");
    let postgres_conn_string = str::replace(conn_string, &db, "postgres");
    let connection = PgConnection::establish(&postgres_conn_string).unwrap();

    connection
        .execute(&format!("CREATE DATABASE \"{}\"", db))
        .map(|_i| ())
}

fn migrate_db(matches: &ArgMatches) {
    let conn_string = matches
        .value_of("connection")
        .expect("Connection string was not provided");

    match create_db(conn_string) {
        Ok(_o) => println!("Creating database"),
        Err(_e) => println!("Database already exists"),
    }
    println!("Migrating database");

    let connection = PgConnection::establish(conn_string).unwrap();

    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
        .expect("Migration failed");
}

fn create_db_and_user(matches: &ArgMatches) {
    let conn_string = matches
        .value_of("connection")
        .expect("Connection string was not provided");

    create_db(conn_string)
        .expect("Can't create database because one with the same name already exists");

    {
        let connection = PgConnection::establish(conn_string).unwrap();

        embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
            .expect("Migration failed");
    }

    let username = matches.value_of("email").expect("Email was not provided");
    let phone = matches
        .value_of("phone")
        .expect("Phone number was not provided");
    let password = matches
        .value_of("password")
        .expect("Password was not provided");
    println!("Creating user");

    let db_connection = DatabaseConnection::new(conn_string).unwrap();
    let user = models::User::create("System", "Administrator", username, phone, password)
        .commit(&db_connection)
        .expect("Failed to create system admin");
    user.add_role(Roles::Admin, &db_connection)
        .expect("Could not assign System Administrator role to the user");
}
