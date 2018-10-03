use lettre::smtp::ConnectionReuseParameters;
use lettre::smtp::SmtpTransportBuilder;
use lettre::{ClientSecurity, EmailTransport, SmtpTransport as LettreSmtpTransport};
use lettre_email::Email;
use mail::transports::Transport;
use std::any::Any;

#[derive(Clone)]
pub struct SmtpTransport {
    domain: String,
    host: String,
    port: u16,
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
    pub fn new(domain: &str, host: &str, port: u16) -> Self {
        SmtpTransport {
            domain: domain.to_string(),
            host: host.to_string(),
            port,
        }
    }

    fn build_smtp_transport(&self) -> LettreSmtpTransport {
        SmtpTransportBuilder::new(
            (self.host.clone().as_str(), self.port),
            ClientSecurity::None,
        ).expect("Failed to create transport")
        .smtp_utf8(true)
        .connection_reuse(ConnectionReuseParameters::NoReuse)
        .build()
    }
}
