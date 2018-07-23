use actix_web::{http::Method, App, HttpResponse};
use config::Config;
use controllers::*;
use middleware::auth::AuthMiddleware;
use server::AppState;

pub fn routes(config: &Config, app: App<AppState>) -> App<AppState> {
    let auth_middleware = AuthMiddleware::new(&config.token_secret);

    let mw = AuthMiddleware::new(&config.token_secret);
    let mw2 = mw.clone();
    let mw3 = mw.clone();
    let mw4 = mw.clone();
    let mw5 = mw.clone();
    let mw6 = mw.clone();
    let mw7 = mw.clone();
    let mw8 = mw.clone();
    //todo look at a better way to do this

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
        .resource("/venues", |r| {
            r.middleware(mw3);
            r.method(Method::GET).with(venues::index);
            r.method(Method::GET).with(venues::show_from_organizations);
            r.method(Method::POST).with(venues::create);
        })
        .resource("/venues/{id}", |r| {
            r.middleware(mw4);
            r.method(Method::GET).with(venues::show);
            r.method(Method::PUT).with(venues::update);
            r.method(Method::PUT).with(venues::add_to_organization);
        })
        .resource("/organizations", |r| {
            r.middleware(mw5);
            r.method(Method::GET).with(organizations::index);
            r.method(Method::POST).with(organizations::create);
        })
        .resource("/organizations/{id}", |r| {
            r.middleware(mw6);
            r.method(Method::GET).with(organizations::show);
            r.method(Method::PUT).with(organizations::update);
        })
        .resource("/events", |r| {
            r.middleware(mw7);
            r.method(Method::GET).with(events::index);
            r.method(Method::GET).with(events::show_from_organizations);
            r.method(Method::GET).with(events::show_from_venues);
            r.method(Method::POST).with(events::create);
        })
        .resource("/events/{id}", |r| {
            r.middleware(mw8);
            r.method(Method::GET).with(events::show);
            r.method(Method::PUT).with(events::update);
        })
        .route("/auth/token", Method::POST, auth::token)
        .route("/login", Method::POST, sessions::create)
        .route("/logout", Method::DELETE, sessions::destroy)
}
