pub mod database;
pub mod test_request;

use actix_web::Body::Binary;
use actix_web::HttpResponse;
use bigneon_api::auth::user::User as AuthUser;
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{Roles, User};
use std::str;
use support::database::TestDatabase;

pub fn unwrap_body_to_string(response: &HttpResponse) -> Result<&str, &'static str> {
    match response.body() {
        Binary(binary) => Ok(str::from_utf8(binary.as_ref()).unwrap()),
        _ => Err("Unexpected response body"),
    }
}

pub fn create_auth_user(role: Roles, database: &TestDatabase) -> AuthUser {
    let user_for_auth = database.create_user().finish();
    create_auth_user_from_user(&user_for_auth, role, database)
}

pub fn create_auth_user_from_user(user: &User, role: Roles, database: &TestDatabase) -> AuthUser {
    let user = user.add_role(role, &*database.get_connection()).unwrap();
    AuthUser::new(user)
}
