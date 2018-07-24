use lettre_email::Email;
use mail::transports::Transport;
use std::any::Any;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct TestTransport {
    pub sent: Arc<Mutex<Vec<Email>>>,
}

impl TestTransport {
    pub fn new() -> Self {
        TestTransport {
            sent: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Transport for TestTransport {
    fn send(&mut self, email: Email) -> Result<String, String> {
        {
            let mut sent = self.sent.lock().unwrap();
            sent.push(email);
        }
        Ok("Mail succeeded".to_string())
    }

    fn box_clone(&self) -> Box<Transport + Send + Sync> {
        Box::new((*self).clone())
    }

    fn as_any(&self) -> &Any {
        self
    }
}
