use dotenv::dotenv;
use std::env;

pub enum Environment {
    Development,
    Test,
    Production,
}

pub struct Config {
    pub cookie_secret_key: String,
    pub database_url: String,
    pub environment: Environment,
}

const DATABASE_URL: &str = "DATABASE_URL";
const TEST_DATABASE_URL: &str = "TEST_DATABASE_URL";
const COOKIE_SECRET_KEY: &str = "COOKIE_SECRET_KEY";

impl Config {
    pub fn new(environment: Environment) -> Config {
        dotenv().ok();

        let database_url = match environment {
            Environment::Test => {
                env::var(&TEST_DATABASE_URL).expect(&format!("{} must be defined.", DATABASE_URL))
            }
            _ => env::var(&DATABASE_URL).expect(&format!("{} must be defined.", DATABASE_URL)),
        };

        let cookie_secret_key =
            env::var(&COOKIE_SECRET_KEY).expect(&format!("{} must be defined.", COOKIE_SECRET_KEY));

        Config {
            cookie_secret_key: cookie_secret_key,
            database_url: database_url,
            environment: environment,
        }
    }
}
