use actix_web::middleware::cors::CorsBuilder;
use actix_web::{http::header, http::Method, App, HttpResponse};
use controllers::*;
use middleware::auth::AuthMiddleware;
use server::AppState;

pub fn routes(app: &mut CorsBuilder<AppState>) -> App<AppState> {
    app.resource("/status", |r| r.get().f(|_| HttpResponse::Ok()))
        .resource("/artists/{id}", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(artists::show);
            r.method(Method::PUT).with(artists::update);
            r.method(Method::DELETE).with(artists::destroy);
        })
        .resource("/artists", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(artists::index);
            r.method(Method::POST).with(artists::create);
        })
        .resource("/venues", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(venues::index);
            r.method(Method::GET).with(venues::show_from_organizations);
            r.method(Method::POST).with(venues::create);
        })
        .resource("/venues/{id}", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(venues::show);
            r.method(Method::PUT).with(venues::update);
            r.method(Method::PUT).with(venues::add_to_organization);
        })
        .resource("/organizations", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(organizations::index);
            r.method(Method::POST).with(organizations::create);
        })
        .resource("/organizations/{id}", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(organizations::show);
            r.method(Method::PUT).with(organizations::update);
        })
        .resource("/events", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(events::index);
            r.method(Method::GET).with(events::show_from_organizations);
            r.method(Method::GET).with(events::show_from_venues);
            r.method(Method::POST).with(events::create);
        })
        .resource("/events/{id}", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(events::show);
            r.method(Method::PUT).with(events::update);
        })
        .resource("/password_reset", |r| {
            r.method(Method::POST).with(password_resets::create);
            r.method(Method::PUT).with(password_resets::update);
        })
        .resource("/users/me", |r| {
            r.middleware(AuthMiddleware::new());
            r.method(Method::GET).with(users::current_user)
        })
        .resource("/auth/token", |r| r.method(Method::POST).with(auth::token))
        .register()
        .default_resource(|r| {
            r.method(Method::GET).f(|_req| {
                HttpResponse::NotFound()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(json!({"error": "Not found"}).to_string())
            });
        })
}
