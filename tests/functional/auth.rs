use actix_web::{http::StatusCode, Json};
use bigneon_api::auth::{claims::AccessToken, claims::RefreshToken, TokenResponse};
use bigneon_api::controllers::auth;
use bigneon_api::controllers::auth::{LoginRequest, RefreshRequest};
use bigneon_api::controllers::users;
use bigneon_api::database::ConnectionGranting;
use bigneon_api::models::register_request::RegisterRequest;
use bigneon_db::models::User;
use crypto::sha2::Sha256;
use jwt::{Header, Token};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
fn token() {
    let database = TestDatabase::new();

    let user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(LoginRequest::new("fake@localhost", "strong_password"));

    let response = auth::token((state, json));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let response: TokenResponse = serde_json::from_str(&body).unwrap();

    let access_token = Token::<Header, AccessToken>::parse(&response.access_token).unwrap();
    let refresh_token = Token::<Header, RefreshToken>::parse(&response.refresh_token).unwrap();

    assert_eq!(access_token.claims.get_id(), user.id);
    assert_eq!(refresh_token.claims.get_id(), user.id);
}

#[test]
fn token_invalid_email() {
    let database = TestDatabase::new();

    let _user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);

    let state = test_request.extract_state();
    let json = Json(LoginRequest::new("incorrect@localhost", "strong_password"));

    let response = auth::token((state, json));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(
        body,
        json!({"error": "Email or password incorrect"}).to_string()
    );
}

#[test]
fn token_incorrect_password() {
    let database = TestDatabase::new();

    let _user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);

    let state = test_request.extract_state();
    let json = Json(LoginRequest::new("incorrect@localhost", "incorrect"));

    let response = auth::token((state, json));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(
        body,
        json!({"error": "Email or password incorrect"}).to_string()
    );
}

#[test]
fn token_refresh() {
    let database = TestDatabase::new();

    let user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);

    let state = test_request.extract_state();
    let refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());
    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(state.config.token_secret.as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response = auth::token_refresh((state, json));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let response: TokenResponse = serde_json::from_str(&body).unwrap();

    let access_token = Token::<Header, AccessToken>::parse(&response.access_token).unwrap();
    assert_eq!(response.refresh_token, refresh_token);
    assert_eq!(access_token.claims.get_id(), user.id);
}

#[test]
fn token_refresh_invalid_refresh_token_secret() {
    let database = TestDatabase::new();

    let user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);

    let state = test_request.extract_state();
    let refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());
    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed("incorrect-secret".as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response = auth::token_refresh((state, json));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, json!({"error": "Invalid token"}).to_string());
}

#[test]
fn token_refresh_invalid_refresh_token() {
    let database = TestDatabase::new();

    let _user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);

    let state = test_request.extract_state();
    let json = Json(RefreshRequest::new(&"not.a.real.token"));

    let response = auth::token_refresh((state, json));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, json!({"error": "Invalid token"}).to_string());
}

#[test]
fn token_refresh_user_does_not_exist() {
    let database = TestDatabase::new();

    let user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);

    let state = test_request.extract_state();
    let mut refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());
    refresh_token_claims.sub = Uuid::new_v4().to_string();

    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(state.config.token_secret.as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response = auth::token_refresh((state, json));

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn token_refresh_password_reset_since_issued() {
    let database = TestDatabase::new();

    let user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();
    let password_modified_timestamp = user.password_modified_at.timestamp() as u64;

    let test_request = TestRequest::create(database);

    let state = test_request.extract_state();
    let mut refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());

    // Issued a second prior to the latest password
    refresh_token_claims.issued = password_modified_timestamp - 1;
    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(state.config.token_secret.as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response = auth::token_refresh((state, json));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, json!({"error": "Invalid token"}).to_string());
}

#[test]
fn token_refreshed_after_password_change() {
    let database = TestDatabase::new();

    let user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();
    let password_modified_timestamp = user.password_modified_at.timestamp() as u64;

    let test_request = TestRequest::create(database);

    let state = test_request.extract_state();
    let mut refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());

    // Issued a second after the latest password
    refresh_token_claims.issued = password_modified_timestamp + 1;
    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(state.config.token_secret.as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response = auth::token_refresh((state, json));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let response: TokenResponse = serde_json::from_str(&body).unwrap();

    let access_token = Token::<Header, AccessToken>::parse(&response.access_token).unwrap();
    assert_eq!(response.refresh_token, refresh_token);
    assert_eq!(access_token.claims.get_id(), user.id);
}

#[test]
fn register_address_exists() {
    let database = TestDatabase::new();

    let _user = User::create("user1", "fake@localhost", "+27112233223", "strong_password")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);
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

    let test_request = TestRequest::create(database);
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
