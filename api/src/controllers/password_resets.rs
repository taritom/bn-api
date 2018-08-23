use actix_web::{HttpResponse, Json, State};
use auth::TokenResponse;
use bigneon_db::models::concerns::users::password_resetable::*;
use bigneon_db::models::User;
use config::Config;
use errors::*;
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
) -> Result<HttpResponse, BigNeonError> {
    if !valid_reset_url(&state.config, &parameters.reset_url) {
        return Ok(HttpResponse::BadRequest().json(json!({
            "error":
                format!(
                    "Invalid `reset_url`: `{}` is not a whitelisted domain",
                    parameters.reset_url
                )
        })));
    }

    let connection = state.database.get_connection();
    let request_pending_response = Ok(HttpResponse::Created().json(json!({
        "message": format!("Your request has been received; {} will receive an email shortly with a link to reset your password if it is an account on file.", parameters.email)
    })));
    let error_message = json!({
        "error": "An error has occurred, please try again later"
    });

    let user = match User::find_by_email(&parameters.email, &*connection) {
        Ok(user) => user,
        Err(_) => return request_pending_response,
    };

    let user = user.create_password_reset_token(&*connection)?;
    let result =
        mailers::user::password_reset_email(&state.config, &user, &parameters.reset_url).deliver();

    match result {
        Ok(_) => request_pending_response,
        Err(_) => Ok(HttpResponse::BadRequest().json(error_message)),
    }
}

pub fn update(
    (state, parameters): (State<AppState>, Json<UpdatePasswordResetParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();

    let user = User::consume_password_reset_token(
        &parameters.password_reset_token,
        &parameters.password,
        &*connection,
    )?;

    Ok(HttpResponse::Ok().json(&TokenResponse::create_from_user(
        &state.token_secret,
        &state.token_issuer,
        &user,
    )))
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
