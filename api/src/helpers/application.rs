use actix_web::{http::StatusCode, HttpResponse};
use validator::ValidationErrors;

pub fn unauthorized() -> HttpResponse {
    unauthorized_with_message("Unauthorized")
}

pub fn unauthorized_with_message(message: &str) -> HttpResponse {
    warn!("Unauthorized: {}", message);
    HttpResponse::Unauthorized().json(json!({"error": message.to_string()}))
}

pub fn forbidden(message: &str) -> HttpResponse {
    warn!("Forbidden: {}", message);
    HttpResponse::Forbidden().json(json!({"error":message.to_string()}))
}

pub fn internal_server_error(message: &str) -> HttpResponse {
    error!("Internal Server Error: {}", message);
    HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
        .into_builder()
        .json(json!({"error": message.to_string()}))
}

pub fn validation_error_response(errors: ValidationErrors) -> HttpResponse {
    HttpResponse::BadRequest()
        .json(json!({"error": "Validation error".to_string(), "fields": errors.inner()}))
}
