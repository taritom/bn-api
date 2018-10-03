use actix_web::{HttpRequest, HttpResponse, Json, State};
use auth::{claims::RefreshToken, TokenResponse};
use bigneon_db::models::User;
use crypto::sha2::Sha256;
use db::Connection;
use errors::*;
use helpers::application;
use jwt::{Header, Token};
use reqwest;
use serde_json;
use server::AppState;
use std::collections::HashMap;

const GOOGLE_RECAPTCHA_SITE_VERIFY_URL: &str = "https://www.google.com/recaptcha/api/siteverify";

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
    #[serde(rename = "g-recaptcha-response")]
    captcha_response: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GoogleCaptchaResponse {
    success: bool,
    #[serde(rename = "error-codes")]
    error_codes: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

impl LoginRequest {
    pub fn new(email: &str, password: &str) -> Self {
        LoginRequest {
            email: String::from(email),
            password: String::from(password),
            captcha_response: None,
        }
    }
}

impl RefreshRequest {
    pub fn new(refresh_token: &str) -> Self {
        RefreshRequest {
            refresh_token: String::from(refresh_token),
        }
    }
}

pub fn token(
    (http_request, connection, login_request): (
        HttpRequest<AppState>,
        Connection,
        Json<LoginRequest>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let state = http_request.state();
    let connection_info = http_request.connection_info();
    let remote_ip = connection_info.remote();

    if let Some(ref google_recaptcha_secret_key) = state.config.google_recaptcha_secret_key {
        match login_request.captcha_response {
            Some(ref captcha_response) => {
                if !verify_google_captcha_response(
                    google_recaptcha_secret_key,
                    captcha_response,
                    remote_ip,
                )? {
                    return application::unauthorized_with_message("Captcha value invalid");
                }
            }
            None => {
                return application::unauthorized_with_message("Captcha required");
            }
        }
    }

    // Generic messaging to prevent exposing user is member of system
    let login_failure_messaging = "Email or password incorrect";

    let user = match User::find_by_email(&login_request.email, connection.get()) {
        Ok(u) => u,
        Err(_e) => return application::unauthorized_with_message(login_failure_messaging),
    };

    if !user.check_password(&login_request.password) {
        return application::unauthorized_with_message(login_failure_messaging);
    }

    let response = TokenResponse::create_from_user(&state.token_secret, &state.token_issuer, &user);
    Ok(HttpResponse::Ok().json(response))
}

pub fn token_refresh(
    (state, connection, refresh_request): (State<AppState>, Connection, Json<RefreshRequest>),
) -> Result<HttpResponse, BigNeonError> {
    let token = match Token::<Header, RefreshToken>::parse(&refresh_request.refresh_token) {
        Ok(token) => token,
        Err(_e) => return application::unauthorized_with_message("Invalid token"),
    };

    if token.verify(state.config.token_secret.as_bytes(), Sha256::new()) {
        let user = User::find(token.claims.get_id(), connection.get())?;

        // If the user changes their password invalidate all refresh tokens
        let password_modified_timestamp = user.password_modified_at.timestamp() as u64;
        if password_modified_timestamp <= token.claims.issued {
            let response = TokenResponse::create_from_refresh_token(
                &state.token_secret,
                &state.token_issuer,
                &user.id,
                &refresh_request.refresh_token,
            );
            Ok(HttpResponse::Ok().json(response))
        } else {
            application::unauthorized_with_message("Invalid token")
        }
    } else {
        application::unauthorized_with_message("Invalid token")
    }
}

fn verify_google_captcha_response(
    google_recaptcha_secret_key: &str,
    captcha_response: &str,
    remote_ip: Option<&str>,
) -> Result<bool, BigNeonError> {
    let client = reqwest::Client::new();
    let mut params = HashMap::new();
    params.insert("secret", google_recaptcha_secret_key);
    params.insert("response", captcha_response);

    if let Some(val) = remote_ip {
        params.insert("remoteip", val);
    }

    let response = client
        .post(GOOGLE_RECAPTCHA_SITE_VERIFY_URL)
        .form(&params)
        .send()?
        .text()?;
    let google_captcha_response: GoogleCaptchaResponse = serde_json::from_str(&response)?;
    if let Some(error_codes) = google_captcha_response.error_codes {
        warn!("Google captcha error encountered: {:?}", error_codes);
    }
    Ok(google_captcha_response.success)
}
