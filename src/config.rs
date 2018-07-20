use dotenv::dotenv;
use std::env;

pub enum Environment {
    Development,
    Test,
    Production,
}

pub struct Config {
    pub api_url: String,
    pub api_port: String,
    pub cookie_secret_key: String,
    pub database_url: String,
    pub environment: Environment,
    pub token_secret: String,
    pub token_issuer: String,
}

const API_URL: &str = "API_URL";
const API_PORT: &str = "API_PORT";
const DATABASE_URL: &str = "DATABASE_URL";
const TEST_DATABASE_URL: &str = "TEST_DATABASE_URL";
const COOKIE_SECRET_KEY: &str = "COOKIE_SECRET_KEY";
const TOKEN_SECRET: &str = "TOKEN_SECRET";
const TOKEN_ISSUER: &str = "TOKEN_ISSUER";

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

        let api_url = env::var(&API_URL).unwrap_or("127.0.0.1".to_string());
        let api_port = env::var(&API_PORT).unwrap_or("8088".to_string());

        let token_secret =
            env::var(&TOKEN_SECRET).expect(&format!("{} must be defined.", TOKEN_SECRET));

        let token_issuer =
            env::var(&TOKEN_ISSUER).expect(&format!("{} must be defined.", TOKEN_ISSUER));

        Config {
            api_url,
            api_port,
            cookie_secret_key,
            database_url,
            token_secret: token_secret,
            token_issuer: token_issuer,
            environment,
        }
    }
}
