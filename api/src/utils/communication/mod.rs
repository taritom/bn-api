use bigneon_db::models::enums::*;
use bigneon_db::models::*;
use config::{Config};
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
) -> impl Future<Item = (), Error = BigNeonError> {
    let communication: Communication = match serde_json::from_value(domain_action.payload.clone()) {
        Ok(v) => v,
        Err(e) => return Either::A(future::err(e.into())),
    };

    match config.environment {
        //TODO Maybe remove this environment check and just rely on the BLOCK_EXTERNAL_COMMS .env
        Environment::Test => Either::A(future::ok(())), //Disable communication system when testing
        _ => {
            let res = match config.block_external_comms {
                true => {
                    jlog!(Trace, "Blocked communication", { "communication": communication });

                    Either::A(future::ok(()))
                }
                _ => {
                    let destination_addresses = communication.destinations.get();

                    let future = match communication.comm_type {
                        CommunicationType::EmailTemplate => {
                            if communication.template_id.is_none() {
                                Box::new(future::err(
                                    ApplicationError::new(
                                        "Template ID must be specified when communication type is EmailTemplate"
                                            .to_string(),
                                    )
                                    .into(),
                                ))
                            } else {
                                let template_id = communication.template_id.as_ref().unwrap();

                                // Short circuit logic if communication template and template is blank
                                if template_id == "" {
                                    jlog!(Trace, "Blocked communication, blank template ID", {
                                        "communication": communication
                                    });
                                    Box::new(future::ok(()))
                                } else {
                                    let extra_data = communication.extra_data;
                                    // Check for provider. Sendgrid templates start with "d-".
                                    // TODO: Make a better distinguisher.

                                    if !template_id.starts_with("d-") {

                                            // Customer IO
                                            let extra_data = extra_data.unwrap();

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
                                                Err(e) => return Either::A(future::err(e.into())),
                                            }
                                    }
                                        else {
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

                                }
                            }
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
            };
            res
        }
    }
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
            )))
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
            )))
        }
    };

    template_data.insert("show_start_date".to_string(), start_datetime.date().to_string());
    template_data.insert("show_start_time".to_string(), start_datetime.time().to_string());

    template_data.insert("show_venue_address".to_string(), venue.address.to_string());
    template_data.insert("show_venue_city".to_string(), venue.city.to_string());
    template_data.insert("show_venue_state".to_string(), venue.state.to_string());
    template_data.insert(
        "show_venue_postal_code".to_string(),
        venue.postal_code.to_string(),
    );

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
