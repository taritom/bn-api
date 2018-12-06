use bigneon_db::models::User;
use config::Config;
use diesel::pg::PgConnection;
use errors::*;
use utils::communication::CommAddress;
use utils::communication::Communication;
use utils::communication::CommunicationType;
use utils::communication::TemplateData;

pub fn send_tickets(
    config: &Config,
    email: String,
    sender_user_id: &str,
    num_tickets: u32,
    transfer_key: &str,
    signature: &str,
    from_user: &User,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let receive_tickets_link = format!(
        "{}/tickets/receive?sender_user_id={}&transfer_key={}&num_tickets={}&signature={}",
        config.front_end_url.clone(),
        sender_user_id,
        transfer_key,
        num_tickets,
        signature
    );

    let source = CommAddress::from(config.communication_default_source_email.clone());
    let destinations = CommAddress::from(email);
    let title = "{sender_name} has sent you some tickets".to_string();
    let template_id = config.sendgrid_template_bn_transfer_tickets.clone();
    let mut template_data = TemplateData::new();
    template_data.insert("sender_name".to_string(), from_user.full_name());
    template_data.insert("receive_tickets_link".to_string(), receive_tickets_link);
    Communication::new(
        CommunicationType::EmailTemplate,
        title,
        None,
        Some(source),
        destinations,
        Some(template_id),
        Some(vec![template_data]),
    )
    .queue(conn)
}
