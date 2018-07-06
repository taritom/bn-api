use dotenv::dotenv;
use std::env;

pub enum Environment {
    Development,
    Test,
    Production,
}

pub struct Config {
    pub environment: Environment,
    pub database_url: String,
}

const DATABASE_URL: &str = "DATABASE_URL";
const TEST_DATABASE_URL: &str = "TEST_DATABASE_URL";

impl Config {
    pub fn new(environment: Environment) -> Config {
        dotenv().ok();

        let database_url = match environment {
            Environment::Test => {
                env::var(&TEST_DATABASE_URL).expect(&format!("{} must be defined.", DATABASE_URL))
            }
            _ => env::var(&DATABASE_URL).expect(&format!("{} must be defined.", DATABASE_URL)),
        };

        Config {
            environment: environment,
            database_url: database_url,
        }
    }
}
