use config::{Config, Environment};
use errors::*;
use std::collections::HashMap;
use utils::sendgrid::*;

pub type TemplateData = HashMap<String, String>;

pub enum CommunicationType {
    Email,
    EmailTemplate,
    Sms,
    PushNotification,
}

pub struct CommAddress {
    addresses: Vec<String>,
}

impl CommAddress {
    pub fn new() -> CommAddress {
        CommAddress {
            addresses: Vec::new(),
        }
    }

    pub fn from(address: &String) -> CommAddress {
        CommAddress {
            addresses: vec![address.clone()],
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

pub struct Communication {
    pub comm_type: CommunicationType,
    pub title: String,
    pub body: Option<String>,
    pub source: Option<CommAddress>,
    pub destinations: CommAddress,
    pub template_id: Option<String>,
    pub template_data: Option<Vec<TemplateData>>,
}

impl Communication {
    pub fn new(
        comm_type: CommunicationType,
        title: String,
        body: Option<String>,
        source: Option<CommAddress>,
        destinations: CommAddress,
        template_id: Option<String>,
        template_data: Option<Vec<TemplateData>>,
    ) -> Communication {
        Communication {
            comm_type,
            title,
            body,
            source,
            destinations,
            template_id,
            template_data,
        }
    }

    pub fn send(&self, config: &Config) -> Result<(), BigNeonError> {
        match config.environment {
            //TODO Maybe remove this environment check and just rely on the BLOCK_EXTERNAL_COMMS .env
            Environment::Test => Ok(()), //Disable communication system when testing
            _ => {
                match config.block_external_comms {
                    true => Ok(()), //Disable communication system when block_external_comms is true,
                    _ => {
                        let destination_addresses = self.destinations.get();
                        match self.comm_type {
                            CommunicationType::Email => {
                                if let (Some(source), Some(_body)) =
                                    (self.source.as_ref(), self.body.as_ref())
                                {
                                    let source_address = source.get_first()?;
                                    send_email(
                                        &config.sendgrid_api_key.clone(),
                                        &source_address,
                                        &destination_addresses,
                                        &self.title,
                                        &self.body,
                                    )
                                } else {
                                    Err(ApplicationError::new(
                                        "Email source not specified".to_string(),
                                    ).into())
                                }
                            }
                            CommunicationType::EmailTemplate => {
                                if let (Some(source), Some(template_id), Some(template_data)) = (
                                    self.source.as_ref(),
                                    self.template_id.as_ref(),
                                    self.template_data.as_ref(),
                                ) {
                                    let source_address = source.get_first()?;
                                    send_email_template(
                                        &config.sendgrid_api_key.clone(),
                                        &source_address,
                                        &destination_addresses,
                                        &template_id,
                                        &template_data,
                                    )
                                } else {
                                    Err(ApplicationError::new(
                                        "Email source not specified".to_string(),
                                    ).into())
                                }
                            }
                            CommunicationType::Sms => Err(ApplicationError::new(
                                "SMS communication not implemented".to_string(),
                            ).into()),
                            CommunicationType::PushNotification => Err(ApplicationError::new(
                                "Push notifications not implemented".to_string(),
                            ).into()),
                        }
                    }
                }
            }
        }
    }
}
