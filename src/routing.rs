use actix_web::middleware::cors::CorsBuilder;
use actix_web::{http::header, http::Method, App, HttpResponse};
use controllers::*;
use server::AppState;

pub fn routes(app: &mut CorsBuilder<AppState>) -> App<AppState> {
    app.resource("/status", |r| r.get().f(|_| HttpResponse::Ok()))
        .resource("/artists/{id}", |r| {
            r.method(Method::GET).with(artists::show);
            r.method(Method::PUT).with(artists::update);
            r.method(Method::DELETE).with(artists::destroy);
        })
        .resource("/artists", |r| {
            r.method(Method::GET).with(artists::index);
            r.method(Method::POST).with(artists::create);
        })
        .resource("/venues/{id}/organizations", |r| {
            r.method(Method::POST).with(venues::add_to_organization);
        })
        .resource("/venues/{id}", |r| {
            r.method(Method::GET).with(venues::show);
            r.method(Method::PUT).with(venues::update);
        })
        .resource("/organizations/{id}/venues", |r| {
            r.method(Method::GET).with(venues::show_from_organizations);
        })
        .resource("/venues", |r| {
            r.method(Method::GET).with(venues::index);
            r.method(Method::POST).with(venues::create);
        })
        .resource("/organizations/{id}/owner", |r| {
            r.method(Method::PUT).with(organizations::update_owner);
        })
        .resource("/organizations/invite_user", |r| {
            r.method(Method::POST).with(organization_invites::create);
        })
        .resource("/organizations/accept_invite", |r| {
            r.method(Method::POST)
                .with(organization_invites::accept_request);
        })
        .resource("organizations/decline_invite", |r| {
            r.method(Method::POST)
                .with(organization_invites::decline_request);
        })
        .resource("/organizations/{id}/users", |r| {
            r.method(Method::DELETE).with(organizations::remove_user);
        })
        .resource("/organizations/{id}", |r| {
            r.method(Method::GET).with(organizations::show);
            r.method(Method::PATCH).with(organizations::update);
        })
        .resource("/organizations", |r| {
            r.method(Method::GET).with(organizations::index);
            r.method(Method::POST).with(organizations::create);
        })
        .resource("/venues/{id}/events", |r| {
            r.method(Method::GET).with(events::show_from_venues);
        })
        .resource("/organizations/{id}/events", |r| {
            r.method(Method::GET).with(events::show_from_organizations);
        })
        .resource("/events", |r| {
            r.method(Method::GET).with(events::index);
            r.method(Method::POST).with(events::create);
        })
        .resource("/events/{id}", |r| {
            r.method(Method::GET).with(events::show);
            r.method(Method::PUT).with(events::update);
        })
        .resource("/password_reset", |r| {
            r.method(Method::POST).with(password_resets::create);
            r.method(Method::PUT).with(password_resets::update);
        })
        .resource("/users/me", |r| {
            r.method(Method::GET).with(users::current_user)
        })
        .resource("/users/register", |r| {
            r.method(Method::POST).with(users::register)
        })
        .resource("/users", |r| {
            r.method(Method::GET).with(users::find_via_email);
        })
        .resource("/external/facebook/login", |r| {
            r.method(Method::POST).f(external::facebook::login)
        })
        .resource("/external/facebook/auth_callback", |r| {
            r.name("facebook_callback");
            r.method(Method::GET).f(external::facebook::auth_callback)
        })
        .resource("/external/facebook/web_login", |r| {
            r.method(Method::POST).with(external::facebook::web_login)
        })
        .resource("/auth/token", |r| r.method(Method::POST).with(auth::token))
        .resource("/auth/token/refresh", |r| {
            r.method(Method::POST).with(auth::token_refresh)
        })
        .register()
        .default_resource(|r| {
            r.method(Method::GET).f(|_req| {
                HttpResponse::NotFound()
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(json!({"error": "Not found"}).to_string())
            });
        })
}
