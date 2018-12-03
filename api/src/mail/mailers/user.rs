use bigneon_db::models::User;
use config::Config;
use diesel::PgConnection;
use errors::*;
use mail::mailers::Mailer;
use utils::communication::*;

pub fn user_registered(
    user_first_name: &String,
    user_email: &String,
    config: &Config,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(&config.communication_default_source_email);
    let destinations = CommAddress::from(user_email);
    let title = "BigNeon Registration".to_string();
    let template_id = config.sendgrid_template_bn_user_registered.clone();
    let mut template_data = TemplateData::new();
    template_data.insert("name".to_string(), user_first_name.clone());
    Communication::new(
        CommunicationType::EmailTemplate,
        title,
        None,
        Some(source),
        destinations,
        Some(template_id),
        Some(vec![template_data]),
    ).queue(conn)
}

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
