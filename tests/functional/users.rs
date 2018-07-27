use actix_web::http::StatusCode;
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
