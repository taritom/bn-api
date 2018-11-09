use actix_web::{http::StatusCode, HttpResponse};
use errors::*;
use serde_json;

pub fn unauthorized() -> Result<HttpResponse, BigNeonError> {
    unauthorized_with_message("Unauthorized")
}

pub fn unauthorized_with_message(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Unauthorized: {}", message);
    let error: BigNeonError = AuthError {
        reason: message.to_string(),
    }.into();
    // Error required for triggering middleware rollback
    Ok(HttpResponse::from_error(error.into())
        .into_builder()
        .status(StatusCode::UNAUTHORIZED)
        .json(json!({"error": message.to_string()})))
}

pub fn forbidden(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Forbidden: {}", message);
    let error: BigNeonError = AuthError {
        reason: message.to_string(),
    }.into();
    // Error required for triggering middleware rollback
    Ok(HttpResponse::from_error(error.into())
        .into_builder()
        .status(StatusCode::FORBIDDEN)
        .json(json!({"error": message.to_string()})))
}

pub fn unprocessable(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Unprocessable: {}", message);
    let error: BigNeonError = ApplicationError {
        reason: message.to_string(),
    }.into();
    // Error required for triggering middleware rollback
    Ok(HttpResponse::from_error(error.into())
        .into_builder()
        .status(StatusCode::UNPROCESSABLE_ENTITY)
        .json(json!({"error":message.to_string()})))
}

pub fn internal_server_error(message: &str) -> Result<HttpResponse, BigNeonError> {
    error!("Internal Server Error: {}", message);
    let error: BigNeonError = ApplicationError {
        reason: message.to_string(),
    }.into();
    // Error required for triggering middleware rollback
    Ok(HttpResponse::from_error(error.into())
        .into_builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .json(json!({"error":message.to_string()})))
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
