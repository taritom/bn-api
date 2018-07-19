use actix_web::Json;
use bigneon_api::controllers::sessions::{self, AuthenticationCredentials};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::User;
use serde_json;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
fn create() {
    let database = TestDatabase::new();
    let email = "joe@tari.com";
    let password = "examplePassword";

    let user = User::create(&"Name", &email, &"555-555-5555", &password)
        .commit(&*database.get_connection())
        .unwrap();
    let user_id = user.id.clone();
    let user_expected_json = serde_json::to_string(&user.for_display()).unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(AuthenticationCredentials {
        email: email.clone().to_string(),
        password: password.clone().to_string(),
    });
    let session = test_request.extract_session();
    let response = sessions::create((state, json, session));

    let session = test_request.extract_session();
    let session_user_id = match session.get::<Uuid>("user_id").unwrap() {
        Some(user_id) => user_id,
        None => panic!("User id failed to save in session"),
    };
    assert_eq!(session_user_id, user_id);

    assert!(
        match session.get::<i32>("login_timestamp").unwrap() {
            Some(_timestamp) => true,
            None => false,
        },
        "Expected login timestamp value"
    );

    match response {
        Ok(body) => {
            assert_eq!(body, user_expected_json);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn create_fails_invalid_email() {
    let database = TestDatabase::new();
    let email = "joe@tari.com";
    let password = "examplePassword";

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(AuthenticationCredentials {
        email: email.clone().to_string(),
        password: password.clone().to_string(),
    });
    let session = test_request.extract_session();
    let response = sessions::create((state, json, session));

    let session = test_request.extract_session();
    assert!(
        match session.get::<Uuid>("user_id").unwrap() {
            Some(_user_id) => false,
            None => true,
        },
        "User should not be set on the session"
    );

    assert!(
        match session.get::<i32>("login_timestamp").unwrap() {
            Some(_timestamp) => false,
            None => true,
        },
        "Login timestamp should not be set on the session"
    );

    let expected_body = json!({"error": "Unable to login, please check your credentials and try again."})
        .to_string();
    match response {
        Ok(body) => {
            assert_eq!(body, expected_body);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn create_fails_invalid_password() {
    let database = TestDatabase::new();
    let email = "joe@tari.com";
    let password = "examplePassword";

    User::create(&"Name", &email, &"555-555-5555", &password)
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(AuthenticationCredentials {
        email: email.clone().to_string(),
        password: "invalidPassword".to_string(),
    });
    let session = test_request.extract_session();
    let response = sessions::create((state, json, session));

    let session = test_request.extract_session();
    match session.get::<Uuid>("user_id").unwrap() {
        Some(_user_id) => panic!("User id found but none expected"),
        None => (),
    };

    let expected_body = json!({"error": "Unable to login, please check your credentials and try again."})
        .to_string();
    match response {
        Ok(body) => {
            assert_eq!(body, expected_body);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn destroy() {
    let database = TestDatabase::new();
    let user = User::create(
        &"Name",
        &"joe@tari.com",
        &"555-555-5555",
        &"examplePassword",
    ).commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);
    let session = test_request.extract_session();
    session.set("user_id", user.id.clone()).unwrap();
    assert!(session.get::<Uuid>("user_id").unwrap().is_some());

    let response = sessions::destroy(session);
    let session = test_request.extract_session();
    assert!(session.get::<Uuid>("user_id").unwrap().is_none());

    match response {
        Ok(body) => {
            assert_eq!(body, "{}");
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn destroy_already_logged_out() {
    let database = TestDatabase::new();
    let test_request = TestRequest::create(database);
    let session = test_request.extract_session();

    let response = sessions::destroy(session);
    let session = test_request.extract_session();
    assert!(session.get::<Uuid>("user_id").unwrap().is_none());

    match response {
        Ok(body) => {
            assert_eq!(body, "{}");
        }
        _ => panic!("Unexpected response body"),
    }
}
