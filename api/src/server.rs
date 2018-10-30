use actix_web::http;
use actix_web::middleware::cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{server, App};
use config::Config;
use db::*;
use middleware::*;
use routing;
use utils::ServiceLocator;

pub struct AppState {
    pub config: Config,
    pub database: Database,
    pub service_locator: ServiceLocator,
}

impl AppState {
    pub fn new(config: Config) -> AppState {
        AppState {
            database: Database::from_config(&config),
            service_locator: ServiceLocator::new(&config),
            config,
        }
    }
}
pub struct Server {
    pub config: Config,
}

impl Server {
    pub fn start(config: Config) {
        let bind_addr = format!("{}:{}", config.api_url, config.api_port);
        info!("Listening on {}", bind_addr);
        server::new({
            move || {
                App::with_state(AppState::new(config.clone()))
                    .middleware(DatabaseTransaction::new())
                    .middleware(Logger::new(
                        r#"{"remote_ip":"%a", "user_agent": "%{User-Agent}i", "request": "%r",
                        "status_code": %s, "response_time": %D}"#,
                    )).configure(|a| {
                        let mut cors_config = Cors::for_app(a);
                        match config.allowed_origins.as_ref() {
                            "*" => cors_config.send_wildcard(),
                            _ => cors_config.allowed_origin(&config.allowed_origins),
                        };
                        cors_config
                            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
                            .allowed_headers(vec![
                                http::header::AUTHORIZATION,
                                http::header::ACCEPT,
                            ]).allowed_header(http::header::CONTENT_TYPE)
                            .max_age(3600);

                        routing::routes(&mut cors_config)
                    })
            }
        }).keep_alive(server::KeepAlive::Tcp(10))
        .bind(&bind_addr)
        .unwrap_or_else(|_| panic!("Can not bind to {}", bind_addr))
        .run();
    }
}
