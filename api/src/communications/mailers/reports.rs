use bigneon_db::models::*;
use config::Config;
use diesel::PgConnection;
use errors::*;
use models::*;
use serde_json;
use std::collections::HashMap;

pub fn ticket_counts(
    email: String,
    event: &Event,
    ticket_count_report: &TicketCountReport,
    config: &Config,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    let source = CommAddress::from(config.communication_default_source_email.clone());
    let destinations = CommAddress::from(email);
    let title = "BigNeon Ticket Counts".to_string();
    let template_id = config.email_templates.ticket_count_report.to_string();
    let mut template_data: HashMap<String, serde_json::Value> = HashMap::new();
    DomainEvent::webhook_payload_event_data(&event, &mut template_data, conn)?;
    template_data.insert("report".to_string(), json!(ticket_count_report));

    let mut extra_data: HashMap<String, String> = HashMap::new();
    for (key, value) in template_data {
        extra_data.insert(key, value.to_string());
    }

    Communication::new(
        CommunicationType::EmailTemplate,
        title,
        None,
        Some(source),
        destinations,
        Some(template_id),
        None,
        Some(vec!["ticket_counts", "reports"]),
        Some(extra_data),
    )
    .queue(conn)?;

    Ok(())
}
