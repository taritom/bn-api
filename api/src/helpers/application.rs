use actix_web::{http, http::StatusCode, HttpResponse, Responder};
use auth::user::User as AuthUser;
use errors::*;
use serde_json::{self, Value};
use std::collections::HashMap;
use serde::Serialize;
use db::{ConnectionRedis, RedisCommands};
use config::Config;
use std::borrow::Borrow;

pub fn unauthorized<T: Responder>(
    user: Option<AuthUser>,
    additional_data: Option<HashMap<&'static str, Value>>,
) -> Result<T, BigNeonError> {
    unauthorized_with_message("User does not have the required permissions", user, additional_data)
}

pub fn unauthorized_with_message<T: Responder>(
    message: &str,
    auth_user: Option<AuthUser>,
    additional_data: Option<HashMap<&'static str, Value>>,
) -> Result<T, BigNeonError> {
    if let Some(auth_user) = auth_user {
        auth_user.log_unauthorized_access_attempt(additional_data.unwrap_or(HashMap::new()));
    }

    Err(AuthError::new(AuthErrorType::Unauthorized, message.into()).into())
}

pub fn forbidden<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    Err(AuthError::new(AuthErrorType::Forbidden, message.into()).into())
}

pub fn unprocessable<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    Err(ApplicationError::new_with_type(ApplicationErrorType::Unprocessable, message.to_string()).into())
}
pub fn bad_request<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    Err(ApplicationError::new_with_type(ApplicationErrorType::BadRequest, message.to_string()).into())
}

pub fn internal_server_error<T: Responder>(message: &str) -> Result<T, BigNeonError> {
    error!("Internal Server Error: {}", message);
    Err(ApplicationError::new(message.to_string()).into())
}

pub fn no_content() -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::new(StatusCode::NO_CONTENT))
}

pub fn not_found() -> Result<HttpResponse, BigNeonError> {
    warn!("Not found");
    Ok(HttpResponse::new(StatusCode::NOT_FOUND))
}

pub fn created(json: serde_json::Value) -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::new(StatusCode::CREATED).into_builder().json(json))
}

pub fn redirect(url: &str) -> Result<HttpResponse, BigNeonError> {
    Ok(HttpResponse::Found().header(http::header::LOCATION, url).finish())
}

// Redis cache helper functions
pub(crate) fn set_cached_value<T: Serialize>(
    redis_connection: ConnectionRedis,
    config: &Config,
    http_response: &HttpResponse,
    query: &T,
) -> Result<(), BigNeonError> {
    let mut redis_connection = redis_connection.conn()?;
    let body = ConnectionRedis::unwrap_body_to_string(http_response);
    let cache_period = config.cache_period_milli.clone();
    let query_serialized = serde_json::to_string(query)?;
    if let Some(body) = body.ok() {
        if cache_period.is_some() {
            let payload_json = serde_json::to_string(body.borrow())?;
            redis_connection.set_cache_value(query_serialized.borrow(), payload_json.borrow());
        }
    }
    Ok(())
}

pub(crate) fn get_cached_value<T: Serialize>(
    redis_connection: ConnectionRedis,
    config: &Config,
    query: T,
) -> Result<HttpResponse, BigNeonError> {
    let mut redis_connection = redis_connection.conn()?;
    let cache_period = config.cache_period_milli.clone();
    let query_serialized = serde_json::to_string(&query)?;
    // only look for cached value if a cached_period is giving
    cache_period
        .and_then(|period| redis_connection.get_cache_value(query_serialized.borrow(), period as i64))
        .map(|cached_value| Ok(HttpResponse::Ok().json(&cached_value)))
        .unwrap_or_else(|| Err(ApplicationError::new("could not retrieve value from cache".to_string()).into()))
}
