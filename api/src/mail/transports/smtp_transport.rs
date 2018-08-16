use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::extension::ClientId;
use lettre::{EmailTransport, SmtpTransport as LettreSmtpTransport};
use lettre_email::Email;
use mail::transports::Transport;
use std::any::Any;

#[derive(Clone)]
pub struct SmtpTransport {
    domain: String,
    host: String,
    user_name: String,
    password: String,
}

impl Transport for SmtpTransport {
    fn send(&mut self, email: Email) -> Result<String, String> {
        let mut transport = self.build_smtp_transport();

        match transport.send(&email) {
            Ok(_response) => Ok("Mail has sent successfully".to_string()),
            Err(e) => Err(format!("Mail failed to send: {}", e)),
        }
    }

    fn box_clone(&self) -> Box<Transport + Send + Sync> {
        Box::new((*self).clone())
    }

    fn as_any(&self) -> &Any {
        self
    }
}

impl SmtpTransport {
    pub fn new(domain: &str, host: &str, user_name: &str, password: &str) -> Self {
        SmtpTransport {
            domain: domain.clone().to_string(),
            host: host.clone().to_string(),
            user_name: user_name.clone().to_string(),
            password: password.clone().to_string(),
        }
    }

    fn build_smtp_transport(&self) -> LettreSmtpTransport {
        LettreSmtpTransport::simple_builder(&self.host.clone())
            .unwrap()
            .hello_name(ClientId::Domain(self.domain.clone()))
            .credentials(Credentials::new(
                self.user_name.clone(),
                self.password.clone(),
            ))
            .smtp_utf8(true)
            .authentication_mechanism(Mechanism::Plain)
            .build()
    }
}
