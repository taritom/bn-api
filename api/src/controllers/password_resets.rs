use actix_web::{HttpResponse, Json, State};
use auth::TokenResponse;
use bigneon_db::models::concerns::users::password_resetable::*;
use bigneon_db::models::User;
use db::Connection;
use errors::*;
use mail::mailers;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreatePasswordResetParameters {
    pub email: String,
}

#[derive(Deserialize)]
pub struct UpdatePasswordResetParameters {
    pub password_reset_token: Uuid,
    pub password: String,
}

pub fn create(
    (state, connection, parameters): (
        State<AppState>,
        Connection,
        Json<CreatePasswordResetParameters>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let request_pending_response = Ok(HttpResponse::Created().json(json!({
        "message": format!("Your request has been received; {} will receive an email shortly with a link to reset your password if it is an account on file.", parameters.email)
    })));
    let error_message = json!({
        "error": "An error has occurred, please try again later"
    });

    let connection = connection.get();
    let user = match User::find_by_email(&parameters.email, connection) {
        Ok(user) => user,
        Err(_) => return request_pending_response,
    };

    let user = user.create_password_reset_token(connection)?;
    let result = mailers::user::password_reset_email(&state.config, &user).deliver();

    match result {
        Ok(_) => request_pending_response,
        Err(_) => Ok(HttpResponse::BadRequest().json(error_message)),
    }
}

pub fn update(
    (state, connection, parameters): (
        State<AppState>,
        Connection,
        Json<UpdatePasswordResetParameters>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let user = User::consume_password_reset_token(
        &parameters.password_reset_token,
        &parameters.password,
        connection.get(),
    )?;

    Ok(HttpResponse::Ok().json(&TokenResponse::create_from_user(
        &state.token_secret,
        &state.token_issuer,
        &user,
    )))
}
