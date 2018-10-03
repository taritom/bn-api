use actix_web::{http::StatusCode, HttpResponse, Json};
use bigneon_api::auth::{claims::AccessToken, claims::RefreshToken, TokenResponse};
use bigneon_api::controllers::auth;
use bigneon_api::controllers::auth::{LoginRequest, RefreshRequest};
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
    let email = "fake@localhost";
    let password = "strong_password";
    let user = database
        .create_user()
        .with_email(email.to_string())
        .with_password(password.to_string())
        .finish();

    let test_request = TestRequest::create();
    let json = Json(LoginRequest::new("fake@localhost", "strong_password"));

    let response: HttpResponse =
        auth::token((test_request.request, database.connection.into(), json)).into();

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
    database.create_user().finish();

    let test_request = TestRequest::create();
    let json = Json(LoginRequest::new("incorrect@localhost", "strong_password"));

    let response: HttpResponse =
        auth::token((test_request.request, database.connection.into(), json)).into();

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
    let user = database
        .create_user()
        .with_email("fake@localhost".to_string())
        .finish();

    let test_request = TestRequest::create();
    let json = Json(LoginRequest::new(&user.email.unwrap(), "incorrect"));

    let response: HttpResponse =
        auth::token((test_request.request, database.connection.into(), json)).into();

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
    let user = database.create_user().finish();

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());
    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(state.config.token_secret.as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response: HttpResponse =
        auth::token_refresh((state, database.connection.into(), json)).into();
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
    let user = database.create_user().finish();

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());
    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(b"incorrect-secret", Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response: HttpResponse =
        auth::token_refresh((state, database.connection.into(), json)).into();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, json!({"error": "Invalid token"}).to_string());
}

#[test]
fn token_refresh_invalid_refresh_token() {
    let database = TestDatabase::new();
    database.create_user().finish();

    let test_request = TestRequest::create();

    let state = test_request.extract_state();
    let json = Json(RefreshRequest::new(&"not.a.real.token"));

    let response: HttpResponse =
        auth::token_refresh((state, database.connection.into(), json)).into();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, json!({"error": "Invalid token"}).to_string());
}

#[test]
fn token_refresh_user_does_not_exist() {
    let database = TestDatabase::new();

    let user = database.create_user().finish();
    let test_request = TestRequest::create();

    let state = test_request.extract_state();
    let mut refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());
    refresh_token_claims.sub = Uuid::new_v4().to_string();

    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(state.config.token_secret.as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response: HttpResponse =
        auth::token_refresh((state, database.connection.into(), json)).into();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn token_refresh_password_reset_since_issued() {
    let database = TestDatabase::new();

    let user = database.create_user().finish();
    let password_modified_timestamp = user.password_modified_at.timestamp() as u64;

    let test_request = TestRequest::create();

    let state = test_request.extract_state();
    let mut refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());

    // Issued a second prior to the latest password
    refresh_token_claims.issued = password_modified_timestamp - 1;
    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(state.config.token_secret.as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response: HttpResponse =
        auth::token_refresh((state, database.connection.into(), json)).into();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, json!({"error": "Invalid token"}).to_string());
}

#[test]
fn token_refreshed_after_password_change() {
    let database = TestDatabase::new();

    let user = database.create_user().finish();
    let password_modified_timestamp = user.password_modified_at.timestamp() as u64;

    let test_request = TestRequest::create();

    let state = test_request.extract_state();
    let mut refresh_token_claims = RefreshToken::new(&user.id, state.token_issuer.clone());

    // Issued a second after the latest password
    refresh_token_claims.issued = password_modified_timestamp + 1;
    let header: Header = Default::default();
    let refresh_token = Token::new(header, refresh_token_claims)
        .signed(state.config.token_secret.as_bytes(), Sha256::new())
        .unwrap();
    let json = Json(RefreshRequest::new(&refresh_token));

    let response: HttpResponse =
        auth::token_refresh((state, database.connection.into(), json)).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let response: TokenResponse = serde_json::from_str(&body).unwrap();

    let access_token = Token::<Header, AccessToken>::parse(&response.access_token).unwrap();
    assert_eq!(response.refresh_token, refresh_token);
    assert_eq!(access_token.claims.get_id(), user.id);
}
