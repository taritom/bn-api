use actix_web::{HttpResponse, Json, State};
use auth::{claims::RefreshToken, TokenResponse};
use bigneon_db::models::User;
use crypto::sha2::Sha256;
use errors::database_error::ConvertToWebError;
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

pub fn token((state, login_request): (State<AppState>, Json<LoginRequest>)) -> HttpResponse {
    let connection = state.database.get_connection();

    // Generic messaging to prevent exposing user is member of system
    let login_failure_messaging = "Email or password incorrect";

    let user = match User::find_by_email(&login_request.email, &*connection) {
        Ok(u) => u,
        Err(_e) => return application::unauthorized_with_message(login_failure_messaging),
    };

    let user = match user {
        Some(u) => u,
        None => return application::unauthorized_with_message(login_failure_messaging),
    };

    if !user.check_password(&login_request.password) {
        return application::unauthorized_with_message(login_failure_messaging);
    }

    let response = TokenResponse::create_from_user(&state.token_secret, &state.token_issuer, &user);
    HttpResponse::Ok().json(response)
}

pub fn token_refresh(
    (state, refresh_request): (State<AppState>, Json<RefreshRequest>),
) -> HttpResponse {
    let connection = state.database.get_connection();

    let token = match Token::<Header, RefreshToken>::parse(&refresh_request.refresh_token) {
        Ok(token) => token,
        Err(_e) => return application::unauthorized_with_message("Invalid token"),
    };

    if token.verify(state.config.token_secret.as_bytes(), Sha256::new()) {
        match User::find(&token.claims.get_id(), &*connection) {
            Ok(user) => {
                // If the user changes their password invalidate all refresh tokens
                let password_modified_timestamp = user.password_modified_at.timestamp() as u64;
                if password_modified_timestamp <= token.claims.issued {
                    let response = TokenResponse::create_from_refresh_token(
                        &state.token_secret,
                        &state.token_issuer,
                        &user.id,
                        &refresh_request.refresh_token,
                    );
                    HttpResponse::Ok().json(response)
                } else {
                    return application::unauthorized_with_message("Invalid token");
                }
            }
            Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
        }
    } else {
        application::unauthorized_with_message("Invalid token")
    }
}
