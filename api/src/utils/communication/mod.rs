use bigneon_db::models::enums::*;
use bigneon_db::models::*;
use config::Config;
use diesel::PgConnection;
use errors::*;
use futures::future::Either;
use futures::Future;
use itertools::Itertools;
use log::Level::Trace;
use std::collections::HashMap;
use tokio::prelude::*;
use utils::expo;
use utils::firebase;
use utils::sendgrid::mail as sendgrid;
use utils::twilio;
use utils::webhook;

pub type TemplateData = HashMap<String, String>;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct CommAddress {
    addresses: Vec<String>,
}

impl CommAddress {
    pub fn new() -> CommAddress {
        CommAddress {
            addresses: Vec::new(),
        }
    }

    pub fn from(address: String) -> CommAddress {
        CommAddress {
            addresses: vec![address],
        }
    }

    pub fn from_vec(addresses: Vec<String>) -> CommAddress {
        CommAddress { addresses }
    }

    pub fn get(&self) -> Vec<String> {
        (self.addresses.clone())
    }

    pub fn get_first(&self) -> Result<String, BigNeonError> {
        if self.addresses.len() >= 1 {
            Ok(self.addresses[0].clone())
        } else {
            Err(
                ApplicationError::new("Minimum of one communication address required".to_string())
                    .into(),
            )
        }
    }

    pub fn push(&mut self, address: &String) {
        self.addresses.push(address.clone());
    }
}

#[derive(Serialize, Deserialize)]
pub struct Communication {
    pub comm_type: CommunicationType,
    pub title: String,
    pub body: Option<String>,
    pub source: Option<CommAddress>,
    pub destinations: CommAddress,
    pub template_id: Option<String>,
    pub template_data: Option<Vec<TemplateData>>,
    pub categories: Option<Vec<String>>,
    pub extra_data: Option<HashMap<String, String>>,
}

impl Communication {
    pub fn new<S: Into<String>>(
        comm_type: CommunicationType,
        title: String,
        body: Option<String>,
        source: Option<CommAddress>,
        destinations: CommAddress,
        template_id: Option<String>,
        template_data: Option<Vec<TemplateData>>,
        categories: Option<Vec<S>>,
        extra_data: Option<HashMap<String, String>>,
    ) -> Communication {
        Communication {
            comm_type,
            title,
            body,
            source,
            destinations,
            template_id,
            template_data,
            categories: categories.map(|c| c.into_iter().map(|c1| c1.into()).collect_vec()),
            extra_data,
        }
    }

    pub fn queue(&self, connection: &PgConnection) -> Result<(), BigNeonError> {
        DomainAction::create(
            None,
            DomainActionTypes::Communication,
            match self.comm_type {
                CommunicationType::Email => Some(CommunicationChannelType::Email),
                CommunicationType::EmailTemplate => Some(CommunicationChannelType::Email),
                CommunicationType::Sms => Some(CommunicationChannelType::Sms),
                CommunicationType::Push => Some(CommunicationChannelType::Push),
                CommunicationType::Webhook => Some(CommunicationChannelType::Webhook),
            },
            json!(self),
            None,
            None,
        )
        .commit(connection)?;
        Ok(())
    }

    pub fn send_async(
        domain_action: &DomainAction,
        config: &Config,
    ) -> impl Future<Item = (), Error = BigNeonError> {
        let communication: Communication =
            match serde_json::from_value(domain_action.payload.clone()) {
                Ok(v) => v,
                Err(e) => return Either::A(future::err(e.into())),
            };
        if config.environment == Environment::Test {
            //TODO Maybe remove this environment check and just rely on the BLOCK_EXTERNAL_COMMS .env
            return Either::A(future::ok(())); //Disable communication system when testing
        };

        if config.block_external_comms {
            jlog!(Trace, "Blocked communication", {
                "communication": communication
            });

            return Either::A(future::ok(()));
        }

        let destination_addresses = communication.destinations.get();

        let future = match communication.comm_type {
            CommunicationType::Email => sendgrid::send_email_async(
                &config.sendgrid_api_key,
                communication.source.as_ref().unwrap().get_first().unwrap(),
                destination_addresses,
                communication.title.clone(),
                communication.body.clone(),
                communication.categories.clone(),
                communication.extra_data.clone(),
            ),
            CommunicationType::EmailTemplate => sendgrid::send_email_template_async(
                &config.sendgrid_api_key,
                communication.source.as_ref().unwrap().get_first().unwrap(),
                &destination_addresses,
                communication.template_id.clone().unwrap(),
                communication.template_data.as_ref().unwrap(),
                communication.categories.clone(),
                communication.extra_data.clone(),
            ),
            CommunicationType::Sms => twilio::send_sms_async(
                &config.twilio_account_id,
                &config.twilio_api_key,
                communication.source.as_ref().unwrap().get_first().unwrap(),
                destination_addresses,
                &communication.body.unwrap_or(communication.title),
            ),
            CommunicationType::Push => {
                let source = match communication.extra_data.map(|data| data["source"].clone()) {
                    Some(s) => s.to_string(),
                    None => "expo".to_string(),
                };

                match source.as_str() {
                    "firebase" => firebase::send_push_notification_async(
                        &config
                            .firebase
                            .as_ref()
                            .map(|f| f.api_key.clone())
                            .unwrap_or("".to_string()),
                        &destination_addresses,
                        &communication.body.unwrap_or(communication.title),
                    ),
                    "expo" | _ => expo::send_push_notification_async(
                        &destination_addresses,
                        &communication.body.unwrap_or(communication.title),
                    ),
                }
            }
            CommunicationType::Webhook => webhook::send_webhook_async(
                &destination_addresses,
                &communication.body.unwrap_or(communication.title),
            ),
        };

        Either::B(future)
    }
}
