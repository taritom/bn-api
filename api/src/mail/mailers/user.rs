use bigneon_db::models::User;
use config::Config;
use mail::mailers::Mailer;

pub fn password_reset_email(config: &Config, user: &User) -> Mailer {
    let password_reset_link = format!(
        "{}/password-reset?token={}",
        config.front_end_url.clone(),
        user.password_reset_token
            .expect("Password reset token is not set")
    );

    let email: &str = user
        .email
        .as_ref()
        .expect("Password reset token is not set");

    Mailer::new(
        config.clone(),
        (email.to_string(), user.full_name()),
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
