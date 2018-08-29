use actix_web::error;
use actix_web::Error as web_error;
use actix_web::HttpResponse;
use bigneon_db::utils::errors::DatabaseError;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeError;
use std::error::Error;
use std::fmt::Debug;
use std::string::ToString;

pub trait ConvertToWebError: Debug + Error + ToString {
    fn create_http_error(&self) -> web_error;

    fn to_response(&self) -> HttpResponse {
        HttpResponse::from_error(self.create_http_error())
    }
}

impl ConvertToWebError for ReqwestError {
    fn create_http_error(&self) -> web_error {
        error!("Reqwest Error: {}", self.description());
        error::ErrorInternalServerError("Internal error")
    }
}

impl ConvertToWebError for SerdeError {
    fn create_http_error(&self) -> web_error {
        error!("SerdeError Error: {}", self.description());
        error::ErrorInternalServerError("Internal error")
    }
}

impl ConvertToWebError for DatabaseError {
    fn create_http_error(&self) -> web_error {
        let new_web_error: web_error = match self.code {
            1000 => error::ErrorBadRequest("Invalid input"),
            1100 => error::ErrorBadRequest("Missing input"),
            2000 => error::ErrorNotFound("No results"),
            3000 => error::ErrorInternalServerError("Query error"),
            3100 => error::ErrorInternalServerError("Could not insert record"),
            3200 => error::ErrorInternalServerError("Could not update record"),
            3300 => error::ErrorInternalServerError("Could not delete record"),
            3400 => error::ErrorConflict("Duplicate record exists"),
            4000 => error::ErrorInternalServerError("Connection error"),
            5000 => error::ErrorInternalServerError("Internal error"),
            _ => error::ErrorInternalServerError("Unknown error"),
        };
        new_web_error
    }
}
