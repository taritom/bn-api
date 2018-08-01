pub mod database;
pub mod test_request;

use actix_web::Body::Binary;
use actix_web::HttpResponse;
use bigneon_api::auth::user::User as AuthUser;
use bigneon_db::db::connections::Connectable;
use bigneon_db::models::{Roles, User};
use std::str;

pub fn unwrap_body_to_string(response: &HttpResponse) -> Result<&str, &'static str> {
    match response.body() {
        Binary(binary) => Ok(str::from_utf8(binary.as_ref()).unwrap()),
        _ => Err("Unexpected response body"),
    }
}

pub fn create_auth_user(role: Roles, connection: &Connectable) -> AuthUser {
    let user_for_auth = User::create("Auth", "auth@tari.com", "555-555-5555", "examplePassword")
        .commit(&*connection)
        .unwrap();
    create_auth_user_from_user(&user_for_auth, role, connection)
}

pub fn create_auth_user_from_user(user: &User, role: Roles, connection: &Connectable) -> AuthUser {
    let user = user.add_role(role, &*connection).unwrap();
    AuthUser::new(user.clone())
}
