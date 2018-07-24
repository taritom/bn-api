use bigneon_db::models::User;
use config::Config;
use mail::mailers::Mailer;

pub fn password_reset_email(config: &Config, user: &User, reset_uri: &str) -> Mailer {
    let password_reset_link = format!(
        "{}?password_reset_token={}",
        reset_uri,
        user.password_reset_token
            .expect("Password reset token is not set")
    );

    Mailer::new(
        config.clone(),
        (user.email.clone(), user.name.clone()),
        (
            config.mail_from_email.clone(),
            config.mail_from_name.clone(),
        ),
        format!("{}: Password reset request", config.app_name),
        format!(
            "This password reset link is valid for 24 hours: {}\nIf you did not request it please ignore this message.",
            password_reset_link
        ),
    )
}
