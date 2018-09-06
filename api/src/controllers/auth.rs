use actix_web::{HttpResponse, Json, State};
use auth::{claims::RefreshToken, TokenResponse};
use bigneon_db::models::User;
use crypto::sha2::Sha256;
use db::Connection;
use errors::*;
use helpers::application;
use jwt::{Header, Token};
use server::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
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
    (state, connection, login_request): (State<AppState>, Connection, Json<LoginRequest>),
) -> Result<HttpResponse, BigNeonError> {
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
