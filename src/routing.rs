use actix_web::{http::Method, App, HttpResponse};
use config::Config;
use controllers::*;
use middleware::auth::AuthMiddleware;
use server::AppState;

pub fn routes(config: &Config, app: App<AppState>) -> App<AppState> {
    let mw = AuthMiddleware::new(&config.token_secret);
    let mw2 = mw.clone();

    app.resource("/status", |r| r.get().f(|_| HttpResponse::Ok()))
        .resource("/artists/{id}", |r| {
            r.middleware(mw);
            r.method(Method::GET).with(artists::show);
            r.method(Method::POST).with(artists::create);
            r.method(Method::PUT).with(artists::update);
            r.method(Method::DELETE).with(artists::destroy);
        })
        .resource("/artists", |r| {
            r.middleware(mw2);
            r.method(Method::GET).with(artists::index);
            r.method(Method::POST).with(artists::create);
        })
        .route("/auth/token", Method::POST, auth::token)
        .route("/login", Method::POST, sessions::create)
        .route("/logout", Method::DELETE, sessions::destroy)
}
