use bigneon_api::config::{Config, Environment};
use bigneon_api::database::ConnectionGranting;
use bigneon_api::mail::mailers;
use bigneon_db::models::concerns::users::password_resetable::PasswordResetable;
use support::database::TestDatabase;

#[test]
fn password_reset_email() {
    let mut config = Config::new(Environment::Test);
    config.mail_from_name = "Big Neon Support".to_string();
    config.mail_from_email = "support@bigneon.com".to_string();
    let database = TestDatabase::new();
    let connection = &*database.get_connection();

    let user = database.create_user().finish();
    let user = user.create_password_reset_token(connection).unwrap();

    let password_reset_email = mailers::user::password_reset_email(&config, &user);
    let name = user.full_name();
    assert_eq!(
        password_reset_email.to(),
        (user.email.unwrap().to_string(), name.to_string())
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
        "{}/reset_password?password_reset_token={}",
        config.front_end_url,
        user.password_reset_token.unwrap()
    )));
}
