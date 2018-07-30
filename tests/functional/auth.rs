use actix_web::{http::StatusCode, Json};
use bigneon_api::auth::big_neon_claims::BigNeonClaims;
use bigneon_api::controllers::auth;
use bigneon_api::controllers::auth::LoginRequest;
use bigneon_api::controllers::users;
use bigneon_api::database::ConnectionGranting;
use bigneon_api::models::register_request::RegisterRequest;
use bigneon_db::models::{Roles, User};
use jwt::Header;
use jwt::Token;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn get_token_valid_data() {
    let database = TestDatabase::new();

    let _user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create_with_route(database, &"/auth/token", &"/auth/token");

    let state = test_request.extract_state();
    let json = Json(LoginRequest::new("fake@localhost", "strong_password"));

    let response = auth::token((state, json));

    match response {
        Ok(body) => {
            let jwt_token = Token::<Header, BigNeonClaims>::parse(&body.token).unwrap();

            assert_eq!(jwt_token.claims.get_roles(), vec![Roles::Guest]);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn register_address_exists() {
    let database = TestDatabase::new();

    let _user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request =
        TestRequest::create_with_route(database, &"/auth/register", &"/auth/register");

    let state = test_request.extract_state();
    let json = Json(RegisterRequest::new(
        &"fake@localhost",
        &"fake@localhost",
        &"555",
        &"not_important",
    ));

    let response = users::register((state, json));

    if response.status() == StatusCode::OK {
        panic!("Duplicate email was allowed when it should not be")
    }
}

#[test]
fn register_succeeds() {
    let database = TestDatabase::new();

    let test_request =
        TestRequest::create_with_route(database, &"/auth/register", &"/auth/register");

    let state = test_request.extract_state();
    let json = Json(RegisterRequest::new(
        &"fake@localhost",
        &"fake@localhost",
        &"555",
        &"not_important",
    ));

    let response = users::register((state, json));

    assert_eq!(response.status(), StatusCode::OK);
}
