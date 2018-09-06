use actix_web::{test, FromRequest, HttpRequest, State};
use bigneon_api::config::{Config, Environment};
use bigneon_api::db::Database;
use bigneon_api::mail::transports::TestTransport;
use bigneon_api::server::AppState;

pub struct TestRequest {
    pub request: HttpRequest<AppState>,
    pub config: Config,
}

impl TestRequest {
    pub fn test_transport(&self) -> &TestTransport {
        self.config
            .mail_transport
            .as_any()
            .downcast_ref::<TestTransport>()
            .unwrap()
    }

    pub fn create() -> TestRequest {
        TestRequest::create_with_uri("/")
    }

    pub fn create_with_uri(path: &str) -> TestRequest {
        let mut config = Config::new(Environment::Test);
        config.token_secret = "test_secret".into();
        config.token_issuer = "bn-api-test".into();

        config.mail_from_email = "support@bigneon.com".to_string();
        config.mail_from_name = "Big Neon".to_string();

        let test_request = test::TestRequest::with_state(AppState {
            config: config.clone(),
            database: Database::from_config(&config),
            token_secret: config.token_secret.clone(),
            token_issuer: config.token_issuer.clone(),
        });

        // TODO: actix-web test requests do not allow router customization except
        // within crate. Forcing an ID here so the extractor can still build the
        // parameters wrapped in the Path struct. Should refactor when they settle
        // on a final test request design as the current does not support extractors.

        let request = test_request
            .param("id", "0f85443e-9e70-45ba-bf28-0f59c183856f")
            .uri(path)
            .finish();
        TestRequest { request, config }
    }

    pub fn extract_state(&self) -> State<AppState> {
        State::<AppState>::extract(&self.request)
    }
}
