use std::fmt;
use std::fmt::Display;

pub struct TariError {
    pub code: u64,
    pub reason: String,
}

impl Display for TariError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error {}: {}", self.code, self.reason)
    }
}

#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ApiResponseSuccess {
    pub success: bool,
    pub id: String,
}
#[derive(Deserialize, Serialize, PartialEq, Debug)]
pub struct ApiResponseFail {
    pub success: bool,
    pub code: u64,
    pub reason: String,
}
