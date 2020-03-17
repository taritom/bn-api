use crate::jwt::Validation;
use crate::support;
use crate::support::database::TestDatabase;
use crate::support::test_request::TestRequest;
use actix_web::{http::StatusCode, HttpResponse};
use bigneon_api::auth::TokenResponse;
use bigneon_api::controllers::password_resets::{self, CreatePasswordResetParameters, UpdatePasswordResetParameters};
use bigneon_api::db::Connection as BigNeonConnection;
use bigneon_api::extractors::*;
use bigneon_db::models::concerns::users::password_resetable::*;
use bigneon_db::models::TokenIssuer;
use bigneon_db::models::User;
use chrono::{Duration, Utc};
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid::Uuid;

#[test]
fn create() {
    let database = TestDatabase::new();
    let email = "joe@tari.com";

    database.create_user().with_email(email.to_string()).finish();
    let expected_json = json!({
        "message": format!("Your request has been received; {} will receive an email shortly with a link to reset your password if it is an account on file.", email)
    })
    .to_string();

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let json = Json(CreatePasswordResetParameters {
        email: email.to_string(),
    });
    let response: HttpResponse = password_resets::create((state, database.connection.clone(), json)).into();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}

#[test]
fn create_fake_email() {
    let database = TestDatabase::new();
    let email = "joe@tari.com";

    let expected_json = json!({
        "message": format!("Your request has been received; {} will receive an email shortly with a link to reset your password if it is an account on file.", email)
    })
    .to_string();

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let json = Json(CreatePasswordResetParameters {
        email: email.to_string(),
    });
    let response: HttpResponse = password_resets::create((state, database.connection, json)).into();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}

#[test]
fn update() {
    let database = TestDatabase::new();
    let connection_object: BigNeonConnection = database.connection.clone().into();

    let user = database.create_user().finish();
    let user = user.create_password_reset_token(database.connection.get()).unwrap();
    let new_password = "newPassword";
    assert!(!user.check_password(&new_password));

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let json = Json(UpdatePasswordResetParameters {
        password_reset_token: user.password_reset_token.unwrap(),
        password: new_password.to_string(),
    });

    let token_issuer = state.config.token_issuer.clone();

    let response: HttpResponse = password_resets::update((state, connection_object, json)).into();

    let user = User::find(user.id, database.connection.get()).unwrap();
    assert!(user.password_reset_token.is_none());
    assert!(user.password_reset_requested_at.is_none());
    assert!(user.check_password(&new_password));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let token_response: TokenResponse = serde_json::from_str(&body).unwrap();
    let refresh_token = token_issuer.decode(&token_response.refresh_token).unwrap();
    let access_token = token_issuer.decode(&token_response.access_token).unwrap();
    assert_eq!(access_token.claims.get_id().unwrap(), user.id);

    let mut validation = Validation::default();
    validation.validate_exp = false;

    assert_eq!(refresh_token.claims.get_id().unwrap(), user.id);
}

#[test]
fn update_expired_token() {
    use bigneon_db::schema::users::dsl::*;
    let database = TestDatabase::new();
    let connection_object: BigNeonConnection = database.connection.clone().into();
    let user = database.create_user().finish();

    let token = Uuid::new_v4();
    let user: User = diesel::update(users.filter(id.eq(user.id)))
        .set(PasswordReset {
            password_reset_token: Some(token),
            password_reset_requested_at: Some(Utc::now().naive_utc() - Duration::days(3)),
        })
        .get_result(database.connection.get())
        .unwrap();
    let new_password = "newPassword";
    assert!(!user.check_password(&new_password));

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let json = Json(UpdatePasswordResetParameters {
        password_reset_token: token,
        password: new_password.to_string(),
    });
    let response: HttpResponse = password_resets::update((state, connection_object, json)).into();

    let user = User::find(user.id, database.connection.get()).unwrap();
    assert_eq!(user.password_reset_token.unwrap(), token);
    assert!(user.password_reset_requested_at.is_some());
    assert!(!user.check_password(&new_password));

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn update_incorrect_token() {
    let database = TestDatabase::new();
    let connection_object: BigNeonConnection = database.connection.clone().into();
    let user = database.create_user().finish();
    let user = user.create_password_reset_token(database.connection.get()).unwrap();
    let new_password = "newPassword";
    let token = user.password_reset_token.unwrap();
    assert!(!user.check_password(&new_password));

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let json = Json(UpdatePasswordResetParameters {
        password_reset_token: Uuid::new_v4(),
        password: new_password.to_string(),
    });
    let response: HttpResponse = password_resets::update((state, connection_object, json)).into();

    let user = User::find(user.id, database.connection.get()).unwrap();
    assert_eq!(user.password_reset_token.unwrap(), token);
    assert!(user.password_reset_requested_at.is_some());
    assert!(!user.check_password(&new_password));

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
