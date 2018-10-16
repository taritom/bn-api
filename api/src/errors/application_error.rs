use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ApplicationError {
    pub reason: String,
}

impl ApplicationError {
    pub fn new(reason: String) -> ApplicationError {
        ApplicationError { reason }
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.reason)
    }
}
impl Error for ApplicationError {}
