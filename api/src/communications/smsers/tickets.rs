use bigneon_db::models::User;
use config::Config;
use diesel::pg::PgConnection;
use errors::*;
use utils::communication::CommAddress;
use utils::communication::Communication;
use utils::communication::CommunicationType;

pub fn send_tickets(
    config: &Config,
    phone: String,
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

    let source = CommAddress::from(config.communication_default_source_phone.clone());
    let destinations = CommAddress::from(phone);
    let body = format!(
        "{} has sent you some tickets: {}",
        from_user.full_name(),
        receive_tickets_link
    );
    Communication::new(
        CommunicationType::Sms,
        body,
        None,
        Some(source),
        destinations,
        None,
        None,
    )
    .queue(conn)
}
