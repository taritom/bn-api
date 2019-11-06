use config::Config;
use customer_io::{CustomerIoClient, Event, EventData};
use errors::BigNeonError;
use futures::future;
use futures::Future;
use std::collections::HashMap;

pub fn send_email_async(
    config: &Config,
    dest_email_addresses: Vec<String>,
    title: String,
    body: Option<String>,
    categories: Option<Vec<String>>,
    unique_args: Option<HashMap<String, String>>,
) -> Box<dyn Future<Item = (), Error = BigNeonError>> {
    let client = match CustomerIoClient::new(
        config.customer_io.api_key.clone(),
        config.customer_io.site_id.clone(),
        &config.customer_io.base_url,
    ) {
        Ok(t) => t,
        Err(err) => return Box::new(future::err(err.into())),
    };

    for address in dest_email_addresses {
        let mut extra_data = HashMap::new();
        extra_data.insert("subject".to_string(), "Test subject".to_string());
        extra_data.insert("message".to_string(), "Test Message".to_string());
        // TODO:    let data = template_data;
        let event = Event {
            name: "general_event_email".to_string(),
            data: EventData {
                recipient: Some("icecool@tari.com".to_string()),
                extra: extra_data,
            },
        };
        client.create_anonymous_event(event).unwrap();
    }

    Box::new(future::ok(()))
}
