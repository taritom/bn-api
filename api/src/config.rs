use dotenv::dotenv;
use std::env;
use tari_client::{HttpTariClient, TariClient, TariTestClient};

#[derive(Clone, PartialEq)]
pub enum Environment {
    Development,
    Test,
    Production,
}

#[derive(Clone)]
pub struct Config {
    pub allowed_origins: String,
    pub front_end_url: String,
    pub api_url: String,
    pub api_port: String,
    pub app_name: String,
    pub database_url: String,
    pub database_pool_size: u32,
    pub domain: String,
    pub environment: Environment,
    pub facebook_app_id: Option<String>,
    pub facebook_app_secret: Option<String>,
    pub google_recaptcha_secret_key: Option<String>,
    pub http_keep_alive: usize,
    pub block_external_comms: bool,
    pub primary_currency: String,
    pub stripe_secret_key: String,
    pub token_secret: String,
    pub token_issuer: String,
    pub tari_client: Box<TariClient + Send + Sync>,
    pub communication_default_source_email: String,
    pub communication_default_source_phone: String,
    pub sendgrid_api_key: String,
    pub sendgrid_template_bn_user_registered: String,
    pub sendgrid_template_bn_purchase_completed: String,
    pub sendgrid_template_bn_org_invite: String,
    pub sendgrid_template_bn_transfer_tickets: String,
    pub sendgrid_template_bn_password_reset: String,
    pub spotify_auth_token: Option<String>,
    pub twilio_account_id: String,
    pub twilio_api_key: String,
}

const ALLOWED_ORIGINS: &str = "ALLOWED_ORIGINS";
const APP_NAME: &str = "APP_NAME";
const API_URL: &str = "API_URL";
const API_PORT: &str = "API_PORT";
const DATABASE_URL: &str = "DATABASE_URL";
const DATABASE_POOL_SIZE: &str = "DATABASE_POOL_SIZE";
const DOMAIN: &str = "DOMAIN";
const FACEBOOK_APP_ID: &str = "FACEBOOK_APP_ID";
const FACEBOOK_APP_SECRET: &str = "FACEBOOK_APP_SECRET";
const GOOGLE_RECAPTCHA_SECRET_KEY: &str = "GOOGLE_RECAPTCHA_SECRET_KEY";
const PRIMARY_CURRENCY: &str = "PRIMARY_CURRENCY";
const STRIPE_SECRET_KEY: &str = "STRIPE_SECRET_KEY";
const TARI_URL: &str = "TARI_URL";
const TEST_DATABASE_URL: &str = "TEST_DATABASE_URL";
const TOKEN_SECRET: &str = "TOKEN_SECRET";
const TOKEN_ISSUER: &str = "TOKEN_ISSUER";
const HTTP_KEEP_ALIVE: &str = "HTTP_KEEP_ALIVE";
// Blocks all external communications from occurring
const BLOCK_EXTERNAL_COMMS: &str = "BLOCK_EXTERNAL_COMMS";
const FRONT_END_URL: &str = "FRONT_END_URL";

//Communication settings
const COMMUNICATION_DEFAULT_SOURCE_EMAIL: &str = "COMMUNICATION_DEFAULT_SOURCE_EMAIL";
const COMMUNICATION_DEFAULT_SOURCE_PHONE: &str = "COMMUNICATION_DEFAULT_SOURCE_PHONE";

//SendGrid settings
const SENDGRID_API_KEY: &str = "SENDGRID_API_KEY";
const SENDGRID_TEMPLATE_BN_USER_REGISTERED: &str = "SENDGRID_TEMPLATE_BN_USER_REGISTERED";
const SENDGRID_TEMPLATE_BN_PURCHASE_COMPLETED: &str = "SENDGRID_TEMPLATE_BN_PURCHASE_COMPLETED";
const SENDGRID_TEMPLATE_BN_ORG_INVITE: &str = "SENDGRID_TEMPLATE_BN_ORG_INVITE";
const SENDGRID_TEMPLATE_BN_TRANSFER_TICKETS: &str = "SENDGRID_TEMPLATE_BN_TRANSFER_TICKETS";
const SENDGRID_TEMPLATE_BN_PASSWORD_RESET: &str = "SENDGRID_TEMPLATE_BN_PASSWORD_RESET";

//Spotify settings
const SPOTIFY_AUTH_TOKEN: &str = "SPOTIFY_AUTH_TOKEN";

const TWILIO_API_KEY: &str = "TWILIO_API_KEY";
const TWILIO_ACCOUNT_ID: &str = "TWILIO_ACCOUNT_ID";

impl Config {
    pub fn new(environment: Environment) -> Self {
        dotenv().ok();

        let app_name = env::var(&APP_NAME).unwrap_or_else(|_| "Big Neon".to_string());

        let database_url = match environment {
            Environment::Test => env::var(&TEST_DATABASE_URL)
                .unwrap_or_else(|_| panic!("{} must be defined.", DATABASE_URL)),
            _ => env::var(&DATABASE_URL)
                .unwrap_or_else(|_| panic!("{} must be defined.", DATABASE_URL)),
        };

        let database_pool_size = env::var(&DATABASE_POOL_SIZE)
            .map(|s| {
                s.parse()
                    .expect("Not a valid integer for database pool size")
            })
            .unwrap_or(20);
        let domain = env::var(&DOMAIN).unwrap_or_else(|_| "api.bigneon.com".to_string());

        let allowed_origins = env::var(&ALLOWED_ORIGINS).unwrap_or_else(|_| "*".to_string());
        let api_url = env::var(&API_URL).unwrap_or_else(|_| "127.0.0.1".to_string());
        let api_port = env::var(&API_PORT).unwrap_or_else(|_| "8088".to_string());

        let primary_currency = env::var(&PRIMARY_CURRENCY).unwrap_or_else(|_| "usd".to_string());
        let stripe_secret_key =
            env::var(&STRIPE_SECRET_KEY).unwrap_or_else(|_| "<stripe not enabled>".to_string());
        let token_secret =
            env::var(&TOKEN_SECRET).unwrap_or_else(|_| panic!("{} must be defined.", TOKEN_SECRET));

        let token_issuer =
            env::var(&TOKEN_ISSUER).unwrap_or_else(|_| panic!("{} must be defined.", TOKEN_ISSUER));

        let facebook_app_id = env::var(&FACEBOOK_APP_ID).ok();

        let facebook_app_secret = env::var(&FACEBOOK_APP_SECRET).ok();

        let front_end_url =
            env::var(&FRONT_END_URL).unwrap_or_else(|_| panic!("Front end url must be defined"));

        let tari_uri =
            env::var(&TARI_URL).unwrap_or_else(|_| panic!("{} must be defined.", TARI_URL));

        let tari_client = match environment {
            Environment::Test => {
                Box::new(TariTestClient::new(tari_uri)) as Box<TariClient + Send + Sync>
            }
            _ => {
                if tari_uri == "TEST" {
                    Box::new(TariTestClient::new(tari_uri)) as Box<TariClient + Send + Sync>
                } else {
                    Box::new(HttpTariClient::new(tari_uri)) as Box<TariClient + Send + Sync>
                }
            }
        };

        let google_recaptcha_secret_key = env::var(&GOOGLE_RECAPTCHA_SECRET_KEY).ok();

        let communication_default_source_email = env::var(&COMMUNICATION_DEFAULT_SOURCE_EMAIL)
            .unwrap_or_else(|_| panic!("{} must be defined.", COMMUNICATION_DEFAULT_SOURCE_EMAIL));
        let communication_default_source_phone = env::var(&COMMUNICATION_DEFAULT_SOURCE_PHONE)
            .unwrap_or_else(|_| panic!("{} must be defined.", COMMUNICATION_DEFAULT_SOURCE_PHONE));

        let sendgrid_api_key = env::var(&SENDGRID_API_KEY)
            .unwrap_or_else(|_| panic!("{} must be defined.", SENDGRID_API_KEY));

        let sendgrid_template_bn_user_registered = env::var(&SENDGRID_TEMPLATE_BN_USER_REGISTERED)
            .unwrap_or_else(|_| {
                panic!("{} must be defined.", SENDGRID_TEMPLATE_BN_USER_REGISTERED)
            });

        let sendgrid_template_bn_purchase_completed =
            env::var(&SENDGRID_TEMPLATE_BN_PURCHASE_COMPLETED).unwrap_or_else(|_| {
                panic!(
                    "{} must be defined.",
                    SENDGRID_TEMPLATE_BN_PURCHASE_COMPLETED
                )
            });
        let sendgrid_template_bn_org_invite = env::var(&SENDGRID_TEMPLATE_BN_ORG_INVITE)
            .unwrap_or_else(|_| panic!("{} must be defined.", SENDGRID_TEMPLATE_BN_ORG_INVITE));
        let sendgrid_template_bn_transfer_tickets =
            env::var(&SENDGRID_TEMPLATE_BN_TRANSFER_TICKETS).unwrap_or_else(|_| {
                panic!("{} must be defined.", SENDGRID_TEMPLATE_BN_TRANSFER_TICKETS)
            });

        let sendgrid_template_bn_password_reset = env::var(&SENDGRID_TEMPLATE_BN_PASSWORD_RESET)
            .unwrap_or_else(|_| panic!("{} must be defined.", SENDGRID_TEMPLATE_BN_PASSWORD_RESET));

        let spotify_auth_token = env::var(&SPOTIFY_AUTH_TOKEN).ok();
        let twilio_api_key = env::var(&TWILIO_API_KEY)
            .unwrap_or_else(|_| panic!("{} must be defined.", TWILIO_API_KEY));;

        let twilio_account_id = env::var(&TWILIO_ACCOUNT_ID)
            .unwrap_or_else(|_| panic!("{} must be defined.", TWILIO_ACCOUNT_ID));;


        let block_external_comms = match env::var(&BLOCK_EXTERNAL_COMMS)
            .unwrap_or_else(|_| "0".to_string())
            .as_str()
        {
            "0" => false,
            _ => true,
        };

        let http_keep_alive = env::var(&HTTP_KEEP_ALIVE)
            .unwrap_or("75".to_string())
            .parse()
            .unwrap();

        Config {
            allowed_origins,
            app_name,
            api_url,
            api_port,
            database_url,
            database_pool_size,
            domain,
            environment,
            facebook_app_id,
            facebook_app_secret,
            google_recaptcha_secret_key,
            http_keep_alive,
            block_external_comms,
            primary_currency,
            stripe_secret_key,
            token_secret,
            token_issuer,
            front_end_url,
            tari_client,
            communication_default_source_email,
            communication_default_source_phone,
            sendgrid_api_key,
            sendgrid_template_bn_user_registered,
            sendgrid_template_bn_purchase_completed,
            sendgrid_template_bn_org_invite,
            sendgrid_template_bn_transfer_tickets,
            sendgrid_template_bn_password_reset,
            spotify_auth_token,
            twilio_api_key,
            twilio_account_id,
        }
    }
}
