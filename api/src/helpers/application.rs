use actix_web::{http::StatusCode, HttpRequest, HttpResponse};
use auth::user::User as AuthUser;
use errors::*;
use log::Level::Warn;
use serde_json;
use server::AppState;
use std::collections::HashMap;

pub fn unauthorized(
    request: &HttpRequest<AppState>,
    user: Option<AuthUser>,
) -> Result<HttpResponse, BigNeonError> {
    unauthorized_with_message("User does not have the required permissions", request, user)
}

pub fn unauthorized_with_message(
    message: &str,
    request: &HttpRequest<AppState>,
    auth_user: Option<AuthUser>,
) -> Result<HttpResponse, BigNeonError> {
    if let Some(auth_user) = auth_user {
        auth_user.log_unauthorized_access_attempt(HashMap::new());
    } else {
        log_unauthorized_access_attempt_from_request(request);
    }

    let error: BigNeonError = AuthError::new(message.into()).into();
    // Error required for triggering middleware rollback
    Ok(HttpResponse::from_error(error.into())
        .into_builder()
        .status(StatusCode::UNAUTHORIZED)
        .json(json!({"error": message.to_string()})))
}

fn log_unauthorized_access_attempt_from_request(request: &HttpRequest<AppState>) {
    let mut logging_data = HashMap::new();
    logging_data.insert(
        "ip_address",
        json!(request.connection_info().remote().map(|i| i.to_string())),
    );
    logging_data.insert("url", json!(request.uri().to_string()));
    logging_data.insert("method", json!(request.method().to_string()));
    jlog!(Warn, "Unauthorized access attempt", logging_data);
}

pub fn forbidden(message: &str) -> Result<HttpResponse, BigNeonError> {
    warn!("Forbidden: {}", message);
    let error: BigNeonError = AuthError::new(message.into()).into();
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
