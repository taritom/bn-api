use actix_web::HttpResponse;
use actix_web::ResponseError;
use bigneon_db::utils::errors::DatabaseError;
use errors::*;
use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct BigNeonError(DatabaseError);

impl From<DatabaseError> for BigNeonError {
    fn from(e: DatabaseError) -> Self {
        BigNeonError(e)
    }
}

impl fmt::Display for BigNeonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.to_string())
    }
}
impl StdError for BigNeonError {
    fn description(&self) -> &str {
        self.0.description()
    }
}

impl ResponseError for BigNeonError {
    fn error_response(&self) -> HttpResponse {
        self.0.to_response()
    }
}
