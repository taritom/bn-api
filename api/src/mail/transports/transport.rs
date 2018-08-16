use lettre_email::Email;
use std::any::Any;

pub trait Transport {
    fn as_any(&self) -> &Any;
    fn send(&mut self, email: Email) -> Result<String, String>;
    fn box_clone(&self) -> Box<Transport + Send + Sync>;
}

impl Clone for Box<Transport + Send + Sync> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}
