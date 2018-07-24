pub mod database;
pub mod test_request;

use actix_web::Body::Binary;
use actix_web::HttpResponse;
use std::str;

pub fn unwrap_body_to_string(response: &HttpResponse) -> Result<&str, &'static str> {
    match response.body() {
        Binary(binary) => Ok(str::from_utf8(binary.as_ref()).unwrap()),
        _ => Err("Unexpected response body"),
    }
}
