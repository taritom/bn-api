use bigneon_db::models::User;
use config::Config;
use mail::mailers::Mailer;

pub fn send_tickets(
    config: &Config,
    email: &str,
    sender_user_id: &str,
    num_tickets: u32,
    transfer_key: &str,
    signature: &str,
    from_user: &User,
) -> Mailer {
    let receive_tickets_link = format!(
        "{}/tickets/receive?sender_user_id={}&transfer_key={}&num_tickets={}&signature={}",
        config.front_end_url.clone(),
        sender_user_id,
        transfer_key,
        num_tickets,
        signature
    );

    println!("Email link:{}", receive_tickets_link);

    Mailer::new(
        config.clone(),
        (email.to_string(), email.to_string()),
        (
            config.mail_from_email.clone(),
            from_user.full_name(),
        ),
        format!("{} has sent you some tickets", from_user.full_name()),
        format!(
            "This link to receive the tickets is valid for 7 days: {}\nIf you did not request it please ignore this message.",
            receive_tickets_link
        ),
    )
}
