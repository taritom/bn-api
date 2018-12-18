use actix_web::{HttpResponse, Json, State};
use auth::user::User as AuthUser;
use bigneon_db::models::concerns::users::password_resetable::PasswordResetable;
use bigneon_db::models::User;
use bigneon_db::utils::errors::Optional;
use communications::mailers;
use db::Connection;
use errors::*;
use helpers::application;
use server::AppState;
use std::str;

#[derive(Deserialize)]
pub struct UserInviteRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: String,
}

pub fn create(
    (state, connection, parameters, _auth_user): (
        State<AppState>,
        Connection,
        Json<UserInviteRequest>,
        AuthUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();

    // User already exists, so no need to invite them
    if let Some(_) = User::find_by_email(parameters.email.as_str(), connection).optional()? {
        return application::created(json!({}));
    }

    let new_user = User::new_for_invite(
        parameters.first_name.clone(),
        parameters.last_name.clone(),
        Some(parameters.email.clone()),
    );

    let user = new_user.commit(connection)?;
    let user = user.create_password_reset_token(connection)?;

    mailers::user::invite_user_email(&state.config, &user).queue(connection)?;

    application::created(json!({}))
}
