use actix_web::{http::StatusCode, HttpResponse, Json};
use bigneon_api::auth::user::User as AuthUser;
use bigneon_api::controllers::users;
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{DisplayUser, Roles, User};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn current_user() {
    let database = TestDatabase::new();
    let db_user = User::create("Jeff", "test@test.com", "555-555-5555", "password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let user = AuthUser::new(db_user.id, vec![Roles::Guest]);

    let response = users::current_user((state, user));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let user: DisplayUser = serde_json::from_str(&body).unwrap();
    assert_eq!(user.name, "Jeff");
    assert_eq!(user.id, db_user.id);
}

pub fn show_from_email(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let db_user = User::create("Jeff", "test@test.com", "555-555-5555", "password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let user = AuthUser::new(db_user.id, vec![role]);
    let json = Json("test@test.com");
    let response = users::find_via_email((state, json, user));
    let display_user: DisplayUser = db_user.into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        let user: DisplayUser = serde_json::from_str(&body).unwrap();
        assert_eq!(user, display_user);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let event_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, event_expected_json);
    }
}
