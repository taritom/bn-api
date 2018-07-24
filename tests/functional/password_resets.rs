use actix_web::{http::StatusCode, Json};
use bigneon_api::controllers::password_resets::{
    self, CreatePasswordResetParameters, UpdatePasswordResetParameters,
};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::concerns::users::password_resetable::*;
use bigneon_db::models::User;
use chrono::{Duration, Utc};
use diesel;
use diesel::prelude::*;
use lettre::SendableEmail;
use serde_json;
use std::str;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
fn create() {
    let database = TestDatabase::new();
    let email = "joe@tari.com";
    let connection = &*database.get_connection();

    let user = User::create(&"Name", &email, &"555-555-5555", &"examplePassword")
        .commit(connection)
        .unwrap();
    let expected_json = json!({
        "message": format!("Your request has been received; {} will receive an email shortly with a link to reset your password if it is an account on file.", email)
    }).to_string();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(CreatePasswordResetParameters {
        email: email.clone().to_string(),
        reset_url: "http://localhost:9090/reset_password".to_string(),
    });
    let response = password_resets::create((state, json));

    // Reload user
    let user = User::find(&user.id, connection).expect("User to reload");
    let mail_transport = test_request.test_transport();

    {
        let sent = mail_transport.sent.lock().unwrap();
        let mail = sent.first().expect("A password reset mail was expected");
        let envelope = mail.envelope();
        let email_body = str::from_utf8(*mail.message()).unwrap();
        assert_eq!(
            format!("{:?}", envelope.to()),
            format!("[EmailAddress(\"{}\")]", email)
        );
        assert_eq!(
            format!("{:?}", envelope.from().unwrap()),
            "EmailAddress(\"support@bigneon.com\")"
        );
        assert!(email_body.contains("This password reset link is valid for 24 hours"));
        assert!(email_body.contains(user.password_reset_token.unwrap().to_string().as_str()));
    }

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
    }).to_string();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(CreatePasswordResetParameters {
        email: email.clone().to_string(),
        reset_url: "http://localhost:9090/reset_password".to_string(),
    });
    let response = password_resets::create((state, json));

    let mail_transport = test_request.test_transport();

    {
        assert_eq!(mail_transport.sent.lock().unwrap().len(), 0);
    }

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}

#[test]
fn create_invalid_reset_uri() {
    let database = TestDatabase::new();
    let email = "joe@tari.com";
    let reset_url = "http://not_whitelisted/reset_password";

    User::create(&"Name", &email, &"555-555-5555", &"examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let expected_json = json!({
        "error":
            format!(
                "Invalid `reset_url`: `{}` is not a whitelisted domain",
                reset_url
            )
    }).to_string();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(CreatePasswordResetParameters {
        email: email.clone().to_string(),
        reset_url: reset_url.to_string(),
    });
    let response = password_resets::create((state, json));

    let mail_transport = test_request.test_transport();

    {
        assert_eq!(mail_transport.sent.lock().unwrap().len(), 0);
    }

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}

#[test]
fn update() {
    let database = TestDatabase::new();
    let connection = &*database.get_connection();
    let user = User::create(&"Joe", &"joe@tari.com", &"555-555-5555", &"pass")
        .commit(connection)
        .unwrap();

    let user = user.create_password_reset_token(connection).unwrap();
    let new_password = "newPassword";
    assert!(!user.check_password(&new_password));

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(UpdatePasswordResetParameters {
        password_reset_token: user.password_reset_token.unwrap().clone(),
        password: new_password.to_string(),
    });
    let session = test_request.extract_session();
    let response = password_resets::update((state, json, session));

    let session = test_request.extract_session();
    let session_user_id = match session.get::<Uuid>("user_id").unwrap() {
        Some(user_id) => user_id,
        None => panic!("User id failed to save in session"),
    };
    assert_eq!(session_user_id, user.id);

    let user = User::find(&session_user_id, connection).unwrap();
    assert!(user.password_reset_token.is_none());
    assert!(user.password_reset_requested_at.is_none());
    assert!(user.check_password(&new_password));

    let expected_json = serde_json::to_string(&user.for_display()).unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}

#[test]
fn update_expired_token() {
    use bigneon_db::schema::users::dsl::*;
    let database = TestDatabase::new();
    let connection = &*database.get_connection();
    let user = User::create(&"Joe", &"joe@tari.com", &"555-555-5555", &"pass")
        .commit(connection)
        .unwrap();

    let token = Uuid::new_v4();
    let user: User = diesel::update(users.filter(id.eq(user.id)))
        .set(PasswordReset {
            password_reset_token: Some(token.clone()),
            password_reset_requested_at: Some(Utc::now().naive_utc() - Duration::days(3)),
        })
        .get_result(connection.get_connection())
        .unwrap();
    let new_password = "newPassword";
    assert!(!user.check_password(&new_password));

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(UpdatePasswordResetParameters {
        password_reset_token: token.clone(),
        password: new_password.to_string(),
    });
    let session = test_request.extract_session();
    let response = password_resets::update((state, json, session));

    let session = test_request.extract_session();

    assert!(
        match session.get::<Uuid>("user_id").unwrap() {
            Some(_user_id) => false,
            None => true,
        },
        "User id should not be present in session"
    );

    let user = User::find(&user.id, connection).unwrap();
    assert_eq!(user.password_reset_token.unwrap(), token);
    assert!(user.password_reset_requested_at.is_some());
    assert!(!user.check_password(&new_password));

    let expected_json = json!({"error": "[5000] Internal error\nCaused by: Password reset token is expired"})
        .to_string();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}

#[test]
fn update_incorrect_token() {
    let database = TestDatabase::new();
    let connection = &*database.get_connection();
    let user = User::create(&"Joe", &"joe@tari.com", &"555-555-5555", &"pass")
        .commit(connection)
        .unwrap();

    let user = user.create_password_reset_token(connection).unwrap();
    let new_password = "newPassword";
    let token = user.password_reset_token.unwrap();
    assert!(!user.check_password(&new_password));

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(UpdatePasswordResetParameters {
        password_reset_token: Uuid::new_v4(),
        password: new_password.to_string(),
    });
    let session = test_request.extract_session();
    let response = password_resets::update((state, json, session));

    let session = test_request.extract_session();

    assert!(
        match session.get::<Uuid>("user_id").unwrap() {
            Some(_user_id) => false,
            None => true,
        },
        "User id should not be present in session"
    );

    let user = User::find(&user.id, connection).unwrap();
    assert_eq!(user.password_reset_token.unwrap(), token);
    assert!(user.password_reset_requested_at.is_some());
    assert!(!user.check_password(&new_password));

    let expected_json =
        json!({"error": "[3000] Query Error\nCaused by: Error loading user, NotFound"}).to_string();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}
