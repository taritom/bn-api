use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct AuthError {
    pub reason: String,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self.reason)
    }
}
impl Error for AuthError {}
