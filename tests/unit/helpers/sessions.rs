use bigneon_api::database::ConnectionGranting;
use bigneon_api::helpers::sessions;
use bigneon_db::models::User;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
fn login_user() {
    let database = TestDatabase::new();
    let user = User::create(&"Name", &"example@tari.com", &"555-555-5555", &"password")
        .commit(&*database.get_connection())
        .unwrap();
    let test_request = TestRequest::create(database);
    let session = test_request.extract_session();
    assert!(
        match session.get::<Uuid>("user_id").unwrap() {
            Some(_user_id) => false,
            None => true,
        },
        "User id should not be set on session"
    );

    sessions::login_user(&session, &user);

    let session_user_id = match session.get::<Uuid>("user_id").unwrap() {
        Some(user_id) => user_id,
        None => panic!("User id failed to save in session"),
    };
    assert_eq!(session_user_id, user.id);
}

#[test]
fn logout_user() {
    let database = TestDatabase::new();
    let user = User::create(&"Name", &"example@tari.com", &"555-555-5555", &"password")
        .commit(&*database.get_connection())
        .unwrap();
    let test_request = TestRequest::create(database);
    let session = test_request.extract_session();
    sessions::login_user(&session, &user);

    assert!(
        match session.get::<Uuid>("user_id").unwrap() {
            Some(_user_id) => true,
            None => false,
        },
        "User id failed to save in session"
    );

    sessions::logout_user(&session);
    assert!(
        match session.get::<Uuid>("user_id").unwrap() {
            Some(_user_id) => false,
            None => true,
        },
        "User id should not be set on session"
    );
}
