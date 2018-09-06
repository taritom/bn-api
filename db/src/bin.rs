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
#[macro_use]
extern crate validator_derive;
extern crate bigneon_db;
extern crate time;
extern crate validator;

mod models;
mod schema;
mod utils;
pub mod validators;

#[allow(unused_imports)]
embed_migrations!("./migrations");

use clap::ArgMatches;
use clap::{App, Arg, SubCommand};
use diesel::connection::SimpleConnection;
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
        ).subcommand(
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
                ).arg(
                Arg::with_name("force").short("f").help("Drops the database if it exists. WARNING! This is NOT REVERSIBLE")
            ),
        ).subcommand(
            SubCommand::with_name("drop")
                .about("Deletes the current database. WARNING! This is NOT REVERSIBLE")
                .arg(
                    Arg::with_name("connection")
                        .short("c")
                        .takes_value(true)
                        .help("Connection string to the database"),
                )
        ).subcommand(
        SubCommand::with_name("seed")
            .about("Populates the database with example data")
            .arg(Arg::with_name("connection")
                     .short("c")
                     .takes_value(true)
                     .help("Connection string to the database")
            )
    ).get_matches();

    match matches.subcommand() {
        ("create", Some(matches)) => create_db_and_user(matches),
        ("drop", Some(matches)) => drop_db(matches),
        ("migrate", Some(matches)) => migrate_db(matches),
        ("seed", Some(matches)) => seed_db(matches),
        _ => unreachable!("The cli parser will prevent reaching here"),
    }
}

fn change_database(conn_string: &str) -> String {
    let last_slash = conn_string
        .rfind('/')
        .expect("Connection string does not conform to <hostname>/<database>");
    format!("{}/postgres", conn_string.get(..last_slash).unwrap())
}

fn get_db(conn_string: &str) -> (String, String) {
    let parts: Vec<&str> = conn_string.split('/').collect();
    let db = parts.last().unwrap();
    let db = str::replace(db, "'", "''");
    let postgres_conn_string = change_database(conn_string);
    (postgres_conn_string, db)
}

fn execute_sql(postgres_conn_string: &str, query: &str) -> Result<(), diesel::result::Error> {
    let connection = PgConnection::establish(&postgres_conn_string).unwrap();
    connection.execute(query).map(|_i| ())
}

fn create_db(conn_string: &str) -> Result<(), diesel::result::Error> {
    let (postgres_conn_string, db) = get_db(conn_string);
    execute_sql(
        &postgres_conn_string,
        &format!("CREATE DATABASE \"{}\"", db),
    )
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

    if matches.is_present("force") {
        drop_db(matches);
    }

    create_db(conn_string)
        .expect("Can't create database because one with the same name already exists");

    {
        let connection = get_connection(conn_string);

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

    let db_connection = get_connection(conn_string);
    let user = models::User::create("System", "Administrator", username, phone, password)
        .commit(&db_connection)
        .expect("Failed to create system admin");
    user.add_role(Roles::Admin, &db_connection)
        .expect("Could not assign System Administrator role to the user");
}

fn seed_db(matches: &ArgMatches) {
    println!("Seeding database");
    let conn_string = matches
        .value_of("connection")
        .expect("Connection string was not provided");

    let db_connection = get_connection(conn_string);

    let seed_query = include_str!("seed_data/seed.sql");
    println!("Seed {}", seed_query);

    db_connection
        .batch_execute(seed_query)
        .expect("Seeding database failed");
}

fn get_connection(connection_string: &str) -> PgConnection {
    PgConnection::establish(&connection_string).expect("Error connecting to DB")
}

fn drop_db(matches: &ArgMatches) {
    let conn_string = matches
        .value_of("connection")
        .expect("Connection string was not provided");
    let (postgres_conn_string, db) = get_db(conn_string);
    println!("Dropping {} from {}", db, postgres_conn_string);
    execute_sql(
        &postgres_conn_string,
        &format!("DROP DATABASE IF EXISTS \"{}\"", db),
    ).expect("Error dropping database");
}
