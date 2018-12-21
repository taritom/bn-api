use actix_web::{http::StatusCode, HttpRequest, HttpResponse, Responder};
use auth::user::User as AuthUser;
use errors::*;
use log::Level::Warn;
use serde_json::{self, Value};
use server::AppState;
use std::collections::HashMap;

pub fn unauthorized<T: Responder>(
    request: &HttpRequest<AppState>,
    user: Option<AuthUser>,
    additional_data: Option<HashMap<&'static str, Value>>,
) -> Result<T, BigNeonError> {
    unauthorized_with_message(
        "User does not have the required permissions",
        request,
        user,
        additional_data,
    )
}

pub fn unauthorized_with_message<T: Responder>(
    message: &str,
    request: &HttpRequest<AppState>,
    auth_user: Option<AuthUser>,
    additional_data: Option<HashMap<&'static str, Value>>,
) -> Result<T, BigNeonError> {
    if let Some(auth_user) = auth_user {
        auth_user.log_unauthorized_access_attempt(additional_data.unwrap_or(HashMap::new()));
    } else {
        log_unauthorized_access_attempt_from_request(request, additional_data);
    }

    Err(AuthError::new(AuthErrorType::Unauthorized, message.into()).into())
}

fn log_unauthorized_access_attempt_from_request(
    request: &HttpRequest<AppState>,
    additional_data: Option<HashMap<&'static str, Value>>,
) {
    let mut logging_data = additional_data.unwrap_or(HashMap::new());
    logging_data.insert(
        "ip_address",
        json!(request.connection_info().remote().map(|i| i.to_string())),
    );
    logging_data.insert("url", json!(request.uri().to_string()));
    logging_data.insert("method", json!(request.method().to_string()));
    jlog!(Warn, "Unauthorized access attempt", logging_data);
}

pub fn forbidden<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    warn!("Forbidden: {}", message);
    Err(AuthError::new(AuthErrorType::Forbidden, message.into()).into())
}

pub fn unprocessable<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    warn!("Unprocessable: {}", message);
    Err(
        ApplicationError::new_with_type(ApplicationErrorType::Unprocessable, message.to_string())
            .into(),
    )
}

pub fn internal_server_error<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    error!("Internal Server Error: {}", message);
    Err(ApplicationError::new(message.to_string()).into())
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
