use std::fmt;
use std::fmt::Display;

pub struct TariClient {}

pub struct TariError {
    pub code: u64,
    pub reason: String,
}

impl Display for TariError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.reason)
    }
}

#[allow(unused_variables)]
impl TariClient {
    pub fn create_asset(
        &self,
        name: &str,
        symbol: &str,
        decimals: u8,
        total_supply: i64,
        issuer: &str,
    ) -> Result<String, TariError> {
        Ok("Test 1".to_string())
    }
}
