use actix_web::{http::StatusCode, HttpResponse};
use errors::*;
use serde_json;
use validator::ValidationErrors;

pub fn unauthorized() -> Result<HttpResponse, BigNeonError> {
    unauthorized_with_message("Unauthorized")
}

pub fn unauthorized_with_message(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Unauthorized: {}", message);
    Ok(HttpResponse::Unauthorized().json(json!({"error": message.to_string()})))
}

pub fn forbidden(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Forbidden: {}", message);
    Ok(HttpResponse::Forbidden().json(json!({"error":message.to_string()})))
}

pub fn unprocessable(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Unprocessible: {}", message);
    Ok(HttpResponse::new(StatusCode::UNPROCESSABLE_ENTITY)
        .into_builder()
        .json(json!({"error":message.to_string()})))
}

pub fn internal_server_error(message: &str) -> Result<HttpResponse, BigNeonError> {
    error!("Internal Server Error: {}", message);
    Ok(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
        .into_builder()
        .json(json!({"error": message.to_string()})))
}

pub fn validation_error_response(errors: ValidationErrors) -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::BadRequest()
        .json(json!({"error": "Validation error".to_string(), "fields": errors.field_errors()})))
}

pub fn no_content() -> Result<HttpResponse, BigNeonError> {
    warn!("No Content");
    Ok(HttpResponse::new(StatusCode::NO_CONTENT))
}

pub fn not_found() -> Result<HttpResponse, BigNeonError> {
    warn!("Not found");
    Ok(HttpResponse::new(StatusCode::NOT_FOUND))
}

pub fn created(json: serde_json::Value) -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::new(StatusCode::CREATED)
        .into_builder()
        .json(json))
}
