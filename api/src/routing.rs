use actix_web::middleware::cors::CorsBuilder;
use actix_web::{http::Method, App, HttpResponse};
use controllers::*;
use server::AppState;

pub fn routes(app: &mut CorsBuilder<AppState>) -> App<AppState> {
    // Please try to keep in alphabetical order
    app.resource("/artists/{id}", |r| {
        r.method(Method::GET).with(artists::show);
        r.method(Method::PUT).with(artists::update);
    }).resource("/artists", |r| {
            r.method(Method::GET).with(artists::index);
            r.method(Method::POST).with(artists::create);
        })
        .resource("/auth/token", |r| r.method(Method::POST).with(auth::token))
        .resource("/auth/token/refresh", |r| {
            r.method(Method::POST).with(auth::token_refresh)
        })
        .resource("/cart", |r| {
            r.method(Method::POST).with(cart::add);
        })
        .resource("/events", |r| {
            r.method(Method::GET).with(events::index);
            r.method(Method::POST).with(events::create);
        })
        .resource("/events/{id}", |r| {
            r.method(Method::GET).with(events::show);
            r.method(Method::PUT).with(events::update);
        })
        .resource("/events/{id}/artist", |r| {
            r.method(Method::POST).with(events::add_artist);
        })
        .resource("/events/{id}/interest", |r| {
            r.method(Method::POST).with(events::add_interest);
            r.method(Method::DELETE).with(events::remove_interest);
        })
        .resource("/events/{id}/tickets", |r| {
            r.method(Method::POST).with(events::create_tickets);
        })
        .resource("/external/facebook/auth_callback", |r| {
            r.name("facebook_callback");
            r.method(Method::GET).f(external::facebook::auth_callback)
        })
        .resource("/external/facebook/login", |r| {
            r.method(Method::POST).f(external::facebook::login)
        })
        .resource("/external/facebook/web_login", |r| {
            r.method(Method::POST).with(external::facebook::web_login)
        })
        .resource("/organizations/accept_invite", |r| {
            r.method(Method::POST)
                .with(organization_invites::accept_request);
        })
        .resource("/organizations/decline_invite", |r| {
            r.method(Method::POST)
                .with(organization_invites::decline_request);
        })
        .resource("/organizations/{id}/users", |r| {
            r.method(Method::DELETE).with(organizations::remove_user);
            r.method(Method::GET)
                .with(organizations::list_organization_members);
        })
        .resource("/organizations/{id}/events", |r| {
            r.method(Method::GET).with(events::show_from_organizations);
        })
        .resource("/organizations/{id}/invite", |r| {
            r.method(Method::POST).with(organization_invites::create);
        })
        .resource("/organizations/{id}/owner", |r| {
            r.method(Method::PUT).with(organizations::update_owner);
        })
        .resource("/organizations/{id}/users", |r| {
            r.method(Method::DELETE).with(organizations::remove_user);
        })
        .resource("/organizations/{id}/venues", |r| {
            r.method(Method::GET).with(venues::show_from_organizations);
        })
        .resource("/organizations/{id}", |r| {
            r.method(Method::GET).with(organizations::show);
            r.method(Method::PATCH).with(organizations::update);
        })
        .resource("/organizations", |r| {
            r.method(Method::GET).with(organizations::index);
            r.method(Method::POST).with(organizations::create);
        })
        .resource("/password_reset", |r| {
            r.method(Method::POST).with(password_resets::create);
            r.method(Method::PUT).with(password_resets::update);
        })
        .resource("/status", |r| r.get().f(|_| HttpResponse::Ok()))
        .resource("/users/me", |r| {
            r.method(Method::GET).with(users::current_user)
        })
        .resource("/users/register", |r| {
            r.method(Method::POST).with(users::register)
        })
        .resource("/users", |r| {
            r.method(Method::GET).with(users::find_by_email);
        })
        .resource("/users/{id}", |r| {
            r.method(Method::GET).with(users::show);
        })
        .resource("/venues/{id}/events", |r| {
            r.method(Method::GET).with(events::show_from_venues);
        })
        .resource("/venues/{id}/organizations", |r| {
            r.method(Method::POST).with(venues::add_to_organization);
        })
        .resource("/venues/{id}", |r| {
            r.method(Method::GET).with(venues::show);
            r.method(Method::PUT).with(venues::update);
        })
        .resource("/venues", |r| {
            r.method(Method::GET).with(venues::index);
            r.method(Method::POST).with(venues::create);
        })
        .register()
        .default_resource(|r| {
            r.method(Method::GET)
                .f(|_req| HttpResponse::NotFound().json(json!({"error": "Not found".to_string()})));
        })
}
