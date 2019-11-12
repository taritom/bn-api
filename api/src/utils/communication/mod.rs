use bigneon_db::models::enums::*;
use bigneon_db::models::*;
use config::{Config, EmailTemplate};
use customer_io;
use diesel::PgConnection;
use errors::*;
use futures::future::Either;
use futures::Future;
use log::Level::Trace;
use std::collections::HashMap;
use tokio::prelude::*;
use utils::expo;
use utils::sendgrid::mail as sendgrid;
use utils::twilio;
use utils::webhook;
use uuid::Uuid;

pub fn send_async(
    domain_action: &DomainAction,
    config: &Config,
    conn: &PgConnection,
) -> impl Future<Item=(), Error=BigNeonError> {
    let communication: Communication = match serde_json::from_value(domain_action.payload.clone()) {
        Ok(v) => v,
        Err(e) => return Either::A(future::err(e.into())),
    };

    if config.environment == Environment::Test {
        return Either::A(future::ok(()));
    }

    if config.block_external_comms {
        jlog!(Trace, "Blocked communication", { "communication": communication });

        return Either::A(future::ok(()));
    };

    let destination_addresses = communication.destinations.get();

    let future = match communication.comm_type {
        CommunicationType::EmailTemplate => {
            send_email_template(domain_action, &config, conn, communication, &destination_addresses)
        }
        CommunicationType::Sms => twilio::send_sms_async(
            &config.twilio_account_id,
            &config.twilio_api_key,
            communication.source.as_ref().unwrap().get_first().unwrap(),
            destination_addresses,
            &communication.body.unwrap_or(communication.title),
        ),
        CommunicationType::Push => expo::send_push_notification_async(
            &destination_addresses,
            &communication.body.unwrap_or(communication.title),
            Some(json!(communication.extra_data.clone())),
        ),
        CommunicationType::Webhook => webhook::send_webhook_async(
            &destination_addresses,
            &communication.body.unwrap_or(communication.title),
            domain_action.main_table_id,
            conn,
            &config,
        ),
    };
    Either::B(future)
}

fn send_email_template(
    domain_action: &DomainAction,
    config: &Config,
    conn: &PgConnection,
    communication: Communication,
    destination_addresses: &Vec<String>,
) -> Box<dyn Future<Item=(), Error=BigNeonError>> {
    if communication.template_id.is_none() {
        return Box::new(future::err(
            ApplicationError::new("Template ID must be specified when communication type is EmailTemplate".to_string())
                .into(),
        ));
    }
    let template_id = communication.template_id.as_ref().unwrap();

    // Short circuit logic if communication template and template is blank
    if template_id == "" {
        jlog!(Trace, "Blocked communication, blank template ID", {
                "communication": communication
            });
        return Box::new(future::ok(()));
    }
    let extra_data = communication.extra_data;
    // Check for provider. Sendgrid templates start with "d-".

    let template  = if !template_id.starts_with("d-") { EmailTemplate { provider:
        EmailProvider::Sendgrid, template_id: template_id.clone() } else {

        template_id.parse()?
    };

        match template.provider {
            EmailProviders::CustomerIo => let extra_data = extra_data.unwrap();

            let event_id = domain_action.main_table_id.unwrap();
            match customer_io_send_email_async(
                config,
                communication.destinations.addresses,
                communication.title,
                communication.body,
                extra_data,
                event_id,
                conn,
            ) {
            Ok(_t) => Box::new(future::ok(())),
            Err(e) => return Box::new(future::err(e.into())),
        }},
        EmailProvider::Sendgrid => {
            // sendgrid
            sendgrid::send_email_template_async(
                &config.sendgrid_api_key,
                communication.source.as_ref().unwrap().get_first().unwrap(),
                &destination_addresses,
                template_id.to_string(),
                communication.template_data.as_ref().unwrap(),
                communication.categories.clone(),
                extra_data,
            )
        }
        // Customer IO

}

pub fn customer_io_send_email_async(
    config: &Config,
    dest_email_addresses: Vec<String>,
    title: String,
    body: Option<String>,
    mut template_data: HashMap<String, String>,
    event_id: Uuid,
    conn: &PgConnection,
) -> Result<(), BigNeonError> {
    // new() try's to parse base url to URL
    let client = customer_io::CustomerIoClient::new(
        config.customer_io.api_key.clone(),
        config.customer_io.site_id.clone(),
        &config.customer_io.base_url,
    )?;

    template_data.insert("subject".to_string(), title);

    if let Some(b) = body {
        template_data.insert("message".to_string(), b);
    }

    let event = Event::find(event_id, conn)?;
    // parse the venue address if venue
    let venue_id = match event.venue_id {
        Some(t) => t,
        None => {
            return Err(BigNeonError::from(ApplicationError::new_with_type(
                ApplicationErrorType::ServerConfigError,
                "event start date is not available".to_owned(),
            )));
        }
    };
    let venue = Venue::find(venue_id, conn)?;

    template_data.insert("show_name".to_string(), event.name.clone());
    template_data.insert("show_venue_name".to_string(), venue.name.clone());
    let start_datetime = match event.event_start {
        Some(t) => t,
        None => {
            return Err(BigNeonError::from(ApplicationError::new_with_type(
                ApplicationErrorType::ServerConfigError,
                "event start date is not available".to_owned(),
            )));
        }
    };

    template_data.insert("show_start_date".to_string(), start_datetime.date().to_string());
    template_data.insert("show_start_time".to_string(), start_datetime.time().to_string());

    template_data.insert("show_venue_address".to_string(), venue.address.to_string());
    template_data.insert("show_venue_city".to_string(), venue.city.to_string());
    template_data.insert("show_venue_state".to_string(), venue.state.to_string());
    template_data.insert("show_venue_postal_code".to_string(), venue.postal_code.to_string());

    // loop dest_email_addresses, each email will be sent different email address
    for email_address in dest_email_addresses {
        let event = customer_io::Event {
            name: "general_event_email".to_string(),
            data: customer_io::EventData {
                recipient: Some(email_address),
                extra: template_data.clone(),
            },
        };
        client.create_anonymous_event(event).unwrap();
    }
    Ok(())
}
