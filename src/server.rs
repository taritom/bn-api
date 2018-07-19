use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::{server, App};
use config::Config;
use database::{ConnectionGranting, Database};
use routing;

pub struct AppState {
    pub database: Box<ConnectionGranting>,
}

pub struct Server {
    pub config: Config,
}

impl Server {
    pub fn start(config: Config) {
        let bind_addr = format!("{}:{}", config.api_url, config.api_port);
        println!("Listening on {}", bind_addr);
        server::new({
            move || {
                routing::route(App::with_state(AppState {
                    database: Box::new(Database::from_config(&config)),
                })).middleware(SessionStorage::new(
                    CookieSessionBackend::private(config.cookie_secret_key.as_bytes())
                        .secure(false),
                ))
            }
        }).bind(&bind_addr)
            .expect(&format!("Can not bind to {}", bind_addr))
            .run();
    }
}
