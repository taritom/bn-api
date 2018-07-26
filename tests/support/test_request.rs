use actix_web::dev::{Resource, ResourceHandler, Router};
use actix_web::{test, FromRequest, HttpRequest, State};
use bigneon_api::config::{Config, Environment};
use bigneon_api::mail::transports::TestTransport;
use bigneon_api::server::AppState;
use support::database::TestDatabase;

pub struct TestRequest {
    pub request: HttpRequest<AppState>,
    pub config: Config,
}

impl TestRequest {
    pub fn create(database: TestDatabase) -> TestRequest {
        TestRequest::create_with_route(database, "/", "/")
    }

    pub fn test_transport(&self) -> &TestTransport {
        self.config
            .mail_transport
            .as_any()
            .downcast_ref::<TestTransport>()
            .unwrap()
    }

    pub fn create_with_route(database: TestDatabase, route: &str, path: &str) -> TestRequest {
        let mut config = Config::new(Environment::Test);
        config.mail_from_email = "support@bigneon.com".to_string();
        config.mail_from_name = "Big Neon".to_string();
        config.whitelisted_domains.insert("localhost".to_string());

        let mut request = test::TestRequest::with_state(AppState {
            config: config.clone(),
            database: Box::new(database),
            token_secret: "test_secret".into(),
            token_issuer: "bn-api-test".into(),
        }).uri(&path)
            .finish();

        // Associate route logic with request for path parameter matching
        let mut routes = Vec::new();
        routes.push((
            Resource::new("", route),
            Some(ResourceHandler::<()>::default()),
        ));
        let (router, _) = Router::new(
            "",
            request.router().unwrap().server_settings().clone(),
            routes,
        );
        assert!(router.recognize(&mut request).is_some());

        TestRequest { request, config }
    }

    pub fn extract_state(&self) -> State<AppState> {
        State::<AppState>::extract(&self.request)
    }
}
