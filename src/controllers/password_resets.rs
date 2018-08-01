use actix_web::{HttpResponse, Json, State};
use auth::TokenResponse;
use bigneon_db::models::concerns::users::password_resetable::*;
use bigneon_db::models::User;
use config::Config;
use errors::database_error::ConvertToWebError;
use mail::mailers;
use server::AppState;
use url::Url;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreatePasswordResetParameters {
    pub email: String,
    pub reset_url: String,
}

#[derive(Deserialize)]
pub struct UpdatePasswordResetParameters {
    pub password_reset_token: Uuid,
    pub password: String,
}

pub fn create(
    (state, parameters): (State<AppState>, Json<CreatePasswordResetParameters>),
) -> HttpResponse {
    if !valid_reset_url(&state.config, &parameters.reset_url) {
        return HttpResponse::BadRequest().json(json!({
            "error":
                format!(
                    "Invalid `reset_url`: `{}` is not a whitelisted domain",
                    parameters.reset_url
                )
        }));
    }

    let connection = state.database.get_connection();
    let request_pending_message = json!({
        "message": format!("Your request has been received; {} will receive an email shortly with a link to reset your password if it is an account on file.", parameters.email)
    });
    let error_message = json!({
        "error": "An error has occurred, please try again later"
    });

    match User::find_by_email(&parameters.email, &*connection) {
        Ok(user) => match user.create_password_reset_token(&*connection) {
            Ok(user) => {
                let result = mailers::user::password_reset_email(
                    &state.config,
                    &user,
                    &parameters.reset_url,
                ).deliver();

                match result {
                    Ok(_success) => HttpResponse::Created().json(request_pending_message),
                    Err(_e) => HttpResponse::BadRequest().json(error_message),
                }
            }
            Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
        },
        Err(_e) => HttpResponse::Created().json(request_pending_message),
    }
}

pub fn update(
    (state, parameters): (State<AppState>, Json<UpdatePasswordResetParameters>),
) -> HttpResponse {
    let connection = state.database.get_connection();

    match User::consume_password_reset_token(
        &parameters.password_reset_token,
        &parameters.password,
        &*connection,
    ) {
        Ok(user) => HttpResponse::Ok().json(&TokenResponse::create_from_user(
            &state.token_secret,
            &state.token_issuer,
            &user,
        )),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

fn valid_reset_url(config: &Config, reset_url: &String) -> bool {
    match Url::parse(reset_url) {
        Ok(parsed_reset_url) => match parsed_reset_url.host_str() {
            Some(host) => config.whitelisted_domains.contains(&host.to_lowercase()),
            _ => false,
        },
        _ => false,
    }
}
