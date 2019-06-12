use bigneon_db::models::*;
use chrono::prelude::*;
use communications::mailers::insert_event_template_data;
use config::Config;
use diesel::pg::PgConnection;
use errors::*;
use itertools::join;
use utils::communication::CommAddress;
use utils::communication::Communication;
use utils::communication::CommunicationType;
use utils::communication::TemplateData;
use uuid::Uuid;

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
        "{}/tickets/transfers/receive?sender_user_id={}&transfer_key={}&num_tickets={}&signature={}",
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

pub fn transfer_drip_reminder(
    email: String,
    transfer: &Transfer,
    event: &Event,
    source_or_destination: SourceOrDestination,
    config: &Config,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(config.communication_default_source_email.clone());
    let destinations = CommAddress::from(email.clone());
    let title = "BigNeon: Ticket transfer reminder".to_string();
    let template_id = config.sendgrid_template_bn_transfer_tickets_drip.clone();
    let transfer_cancel_url = format!(
        "{}/my-events?event_id={}",
        config.front_end_url.clone(),
        event.id,
    );

    let mut template_data = TemplateData::new();
    template_data.insert(
        "header".to_string(),
        transfer.drip_header(event, source_or_destination, true, conn)?,
    );
    template_data.insert("transfer_cancel_url".to_string(), transfer_cancel_url);
    template_data.insert("transfer_id".to_string(), transfer.id.to_string());
    template_data.insert("destination_email".to_string(), email);
    insert_event_template_data(&mut template_data, event, conn)?;

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

pub fn transfer_cancelled_receipt(
    config: &Config,
    email: String,
    from_user: &User,
    transfer_start_date: NaiveDateTime,
    ticket_ids: &[Uuid],
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(config.communication_default_source_email.clone());
    let destinations = CommAddress::from(email);
    let title = "BigNeon: Cancelled ticket transfer".to_string();
    let template_id = config
        .sendgrid_template_bn_cancel_transfer_tickets_receipt
        .clone();
    let mut template_data = TemplateData::new();
    template_data.insert("sender_name".to_string(), from_user.full_name());
    template_data.insert(
        "transfer_start_date".to_string(),
        transfer_start_date.to_string(),
    );
    template_data.insert("ticket_ids".to_string(), join(ticket_ids, ", "));
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

pub fn transfer_cancelled(
    config: &Config,
    email: String,
    from_user: &User,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(config.communication_default_source_email.clone());
    let destinations = CommAddress::from(email);
    let title = "{sender_name} has cancelled their transfer of tickets".to_string();
    let template_id = config.sendgrid_template_bn_cancel_transfer_tickets.clone();
    let mut template_data = TemplateData::new();
    template_data.insert("sender_name".to_string(), from_user.full_name());
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
