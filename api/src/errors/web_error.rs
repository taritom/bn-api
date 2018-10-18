use actix_web::{http::StatusCode, HttpResponse};
use bigneon_db::utils::errors::DatabaseError;
use bigneon_db::utils::errors::ErrorCode::ValidationError;
use diesel::result::Error as DieselError;
use errors::*;
use lettre::smtp::error::Error as SmtpError;
use payments::PaymentProcessorError;
use reqwest::Error as ReqwestError;
use serde_json::Error as SerdeError;
use std::error::Error;
use std::fmt::Debug;
use std::string::ToString;
use stripe::StripeError;
use tari_client::TariError;

pub trait ConvertToWebError: Debug + Error + ToString {
    fn to_response(&self) -> HttpResponse;
}

fn internal_error(message: &str) -> HttpResponse {
    status_code_and_message(StatusCode::INTERNAL_SERVER_ERROR, message)
}

fn unauthorized(message: &str) -> HttpResponse {
    status_code_and_message(StatusCode::UNAUTHORIZED, message)
}

fn status_code_and_message(code: StatusCode, message: &str) -> HttpResponse {
    HttpResponse::new(code)
        .into_builder()
        .json(json!({"error": message.to_string()}))
}

impl ConvertToWebError for Error {
    fn to_response(&self) -> HttpResponse {
        error!("General error: {}", self);
        internal_error("Internal error")
    }
}

impl ConvertToWebError for DieselError {
    fn to_response(&self) -> HttpResponse {
        error!("Diesel error: {}", self);
        internal_error("Internal error")
    }
}

impl ConvertToWebError for ReqwestError {
    fn to_response(&self) -> HttpResponse {
        error!("Reqwest error: {}", self);
        internal_error("Internal error")
    }
}

impl ConvertToWebError for PaymentProcessorError {
    fn to_response(&self) -> HttpResponse {
        error!("Payment Processor error: {}", self);
        internal_error("Internal error")
    }
}

impl ConvertToWebError for StripeError {
    fn to_response(&self) -> HttpResponse {
        error!("Stripe error: {}", self);
        internal_error("Internal error")
    }
}

impl ConvertToWebError for ApplicationError {
    fn to_response(&self) -> HttpResponse {
        error!("Application error: {}", self);
        internal_error("Internal error")
    }
}

impl ConvertToWebError for SmtpError {
    fn to_response(&self) -> HttpResponse {
        error!("SMTP error: {}", self);
        internal_error("Internal error")
    }
}

impl ConvertToWebError for SerdeError {
    fn to_response(&self) -> HttpResponse {
        error!("Serde error: {}", self);
        internal_error("Internal error")
    }
}

impl ConvertToWebError for TariError {
    fn to_response(&self) -> HttpResponse {
        error!("Tari error: {}", self);
        internal_error("Internal error: Problem with the Tari client")
    }
}

impl ConvertToWebError for AuthError {
    fn to_response(&self) -> HttpResponse {
        error!("AuthError error: {}", self.reason);
        unauthorized(&self.reason)
    }
}

impl ConvertToWebError for DatabaseError {
    fn to_response(&self) -> HttpResponse {
        match self.code {
            1000 => status_code_and_message(StatusCode::BAD_REQUEST, "Invalid input"),
            1100 => status_code_and_message(StatusCode::BAD_REQUEST, "Missing input"),
            2000 => status_code_and_message(StatusCode::NOT_FOUND, "No results"),
            3000 => internal_error("Query error"),
            3100 => internal_error("Could not insert record"),
            3200 => internal_error("Could not update record"),
            3300 => internal_error("Could not delete record"),
            3400 => status_code_and_message(StatusCode::CONFLICT, "Duplicate record exists"),
            4000 => internal_error("Connection error"),
            5000 => internal_error("Internal error"),
            7000 => {
                status_code_and_message(StatusCode::UNPROCESSABLE_ENTITY, "Business process error")
            }
            7200 => match &self.error_code {
                ValidationError { errors } => HttpResponse::new(StatusCode::UNPROCESSABLE_ENTITY)
                    .into_builder()
                    .json(json!({"error": "Validation error".to_string(), "fields": errors})),
                _ => HttpResponse::new(StatusCode::UNPROCESSABLE_ENTITY)
                    .into_builder()
                    .json(json!({"error": "Validation error".to_string()})),
            },
            _ => internal_error("Unknown error"),
        }
    }
}
