use bigneon_api::config::{Config, Environment};
use bigneon_api::mail::mailers;
use bigneon_api::utils::communication::CommAddress;
use bigneon_db::models::concerns::users::password_resetable::PasswordResetable;
use support::database::TestDatabase;

#[test]
fn password_reset_email() {
    let mut config = Config::new(Environment::Test);
    config.mail_from_name = "Big Neon Support".to_string();
    config.mail_from_email = "support@bigneon.com".to_string();
    let database = TestDatabase::new();

    let user = database.create_user().finish();
    let user = user
        .create_password_reset_token(database.connection.get())
        .unwrap();

    let password_reset_email = mailers::user::password_reset_email(&config, &user);
    assert_eq!(
        password_reset_email.destinations,
        CommAddress::from(user.email.unwrap().to_string())
    );
    assert_eq!(
        password_reset_email.source,
        Some(CommAddress::from("noreply@bigneon.com".to_string()))
    );
}
