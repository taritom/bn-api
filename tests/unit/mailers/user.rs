use bigneon_api::config::{Config, Environment};
use bigneon_api::database::ConnectionGranting;
use bigneon_api::mail::mailers;
use bigneon_db::models::concerns::users::password_resetable::PasswordResetable;
use bigneon_db::models::User;
use support::database::TestDatabase;

#[test]
fn password_reset_email() {
    let mut config = Config::new(Environment::Test);
    config.mail_from_name = "Big Neon Support".to_string();
    config.mail_from_email = "support@bigneon.com".to_string();
    let database = TestDatabase::new();
    let connection = &*database.get_connection();

    let user = User::create(
        &"Name",
        &"joe@tari.com",
        &"555-555-5555",
        &"examplePassword",
    ).commit(connection)
        .unwrap();
    let user = user.create_password_reset_token(connection).unwrap();

    let reset_uri = "http://localhost/reset";

    let password_reset_email = mailers::user::password_reset_email(&config, &user, reset_uri);

    assert_eq!(
        password_reset_email.to(),
        (user.email.unwrap().to_string(), user.name.to_string())
    );
    assert_eq!(
        password_reset_email.from(),
        (
            "support@bigneon.com".to_string(),
            "Big Neon Support".to_string()
        )
    );
    assert_eq!(
        password_reset_email.subject(),
        "Big Neon: Password reset request".to_string()
    );
    assert!(
        password_reset_email
            .body()
            .contains("This password reset link is valid for 24 hours")
    );
    assert!(password_reset_email.body().contains(&format!(
        "{}?password_reset_token={}",
        reset_uri,
        user.password_reset_token.unwrap()
    )));
}
