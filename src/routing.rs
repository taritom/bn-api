use actix_web::{http::Method, App};
use controllers::*;
use server::AppState;

pub fn route(app: App<AppState>) -> App<AppState> {
    app.resource("/", |r| r.method(Method::GET).f(artists::index))
        .resource("/artists", |r| r.method(Method::GET).f(artists::index))
        .resource("/artists/{id}", |r| {
            r.method(Method::GET).with(artists::show)
        })
        .resource("/artists/{id}", |r| {
            r.method(Method::POST).with(artists::create)
        })
        .resource("/artists/{id}", |r| {
            r.method(Method::PUT).with(artists::update)
        })
        .resource("/artists/{id}", |r| {
            r.method(Method::DELETE).with(artists::destroy)
        })
}
