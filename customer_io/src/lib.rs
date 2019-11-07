use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use url::{ParseError, Url};
use uuid::Uuid;

pub struct CustomerIoClient {
    site_id: String,
    api_key: String,
    base_url: Url,
}

#[derive(Debug, derive_error::Error)]
pub enum CustomerIoError {
    ParseError(ParseError),
    ReqwestError(reqwest::Error),
}

impl CustomerIoClient {
    pub fn new(api_key: String, site_id: String, base_url: &str) -> Result<Self, CustomerIoError> {
        let base_url = Url::parse(base_url)?;
        let client = CustomerIoClient {
            api_key,
            site_id,
            base_url,
        };
        Ok(client)
    }

    pub fn create_event(&self, event: Event, customer_id: Uuid) -> Result<(), CustomerIoError> {
        let url = self
            .base_url
            .join(&format!("customers/{}/events", customer_id))?;
        let response = reqwest::Client::new()
            .post(&url.to_string())
            .basic_auth(&self.site_id, Some(&self.api_key))
            .json(&event)
            .send()?;
        dbg!(&response);
        response.error_for_status()?;
        Ok(())
    }

    pub fn create_anonymous_event(&self, event: Event) -> Result<(), CustomerIoError> {
        let url = self.base_url.join("events")?;
        let response = reqwest::Client::new()
            .post(&url.to_string())
            .basic_auth(&self.site_id, Some(&self.api_key))
            .json(&event)
            .send()?;
        dbg!(&response);
        response.error_for_status()?;
        Ok(())
    }
}

#[derive(Serialize)]
pub struct Event {
    pub name: String,
    pub data: EventData,
}

#[derive(Serialize)]
pub struct EventData {
    pub recipient: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

#[cfg(test)]
mod test {
    use super::*;
    use uuid::UuidVersion;

    #[test]
    fn create_event() {
        let client = CustomerIoClient::new(
            "00b2b4c04c75c464136a".to_string(),
            "3a5a4371136980a53bc3".to_string(),
            "https://track.customer.io/api/v1/",
        )
        .unwrap();
        let mut extra_data = HashMap::new();
        extra_data.insert("subject".to_string(), "Test subject".to_string());
        extra_data.insert("message".to_string(), "Test Message".to_string());
        extra_data.insert("show_event_name".to_string(), "Test Message".to_string());
        extra_data.insert(
            "show_start_date".to_string(),
            "2019-11-15T12:21:11Z".to_string(),
        );
        extra_data.insert("show_start_time".to_string(), "Test Message".to_string());
        extra_data.insert("show_venue_name".to_string(), "Test Message".to_string());
        extra_data.insert("show_venue_address".to_string(), "Test Message".to_string());
        extra_data.insert("show_venue_city".to_string(), "Test Message".to_string());
        extra_data.insert("show_venue_state".to_string(), "Test Message".to_string());
        extra_data.insert(
            "show_venue_postal_code".to_string(),
            "Test Message".to_string(),
        );
        //        if let Some(venue) = event.venue(conn)? {
        //            data.insert("show_venue_address".to_string(), json!(venue.address));
        //            data.insert("show_venue_city".to_string(), json!(venue.city));
        //            data.insert("show_venue_state".to_string(), json!(venue.state));
        //            data.insert("show_venue_country".to_string(), json!(venue.country));
        //            data.insert(
        //                "show_venue_postal_code".to_string(),
        //                json!(venue.postal_code),
        //            );
        //            data.insert(
        //                "show_venue_phone".to_string(),
        //                json!(venue.phone.unwrap_or("".to_string())),
        //            );
        //            data.insert("show_venue_name".to_string(), json!(venue.name));
        //            data.insert("show_timezone".to_string(), json!(venue.timezone));
        //        }

        client
            .create_event(
                Event {
                    name: "general_event_email".to_string(),
                    data: EventData {
                        recipient: Some("icecool@tari.com".to_string()),
                        extra: extra_data,
                    },
                },
                Uuid::new_v4(),
            )
            .unwrap();
        panic!("Asdf");
    }

    #[test]
    fn create_anonymous_event() {
        let client = CustomerIoClient::new(
            "00b2b4c04c75c464136a".to_string(),
            "3a5a4371136980a53bc3".to_string(),
            "https://track.customer.io/api/v1/",
        )
        .unwrap();
        //                client.create_anonymous_event(Event { name: "general_event_email".to_string(),  }).unwrap();
        panic!("Asdf");
    }
}
