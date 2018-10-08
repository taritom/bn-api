pub mod database;
pub mod test_request;

use actix_web::{http::StatusCode, Body::Binary, HttpResponse};
use bigneon_api::auth::user::User as AuthUser;
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
    if role == Roles::Admin || role == Roles::User {
        let user = user.add_role(role, &database.connection).unwrap();
        AuthUser::new(user)
    } else {
        AuthUser::new(user.clone())
    }
}

pub fn expects_unauthorized(response: &HttpResponse) {
    let expected_json: HttpResponse;
    expected_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let expected_text = unwrap_body_to_string(&expected_json).unwrap();
    let body = unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_text);
}

pub fn expects_forbidden(response: &HttpResponse, message: Option<&str>) {
    let expected_json: HttpResponse;
    expected_json = HttpResponse::Forbidden().json(json!({
        "error": message.unwrap_or("You do not have access to this order")
    }));
    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    let expected_text = unwrap_body_to_string(&expected_json).unwrap();
    let body = unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_text);
}
