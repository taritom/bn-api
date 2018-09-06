use actix_web::{HttpResponse, Json, Path, Query};
use auth::user::Scopes;
use auth::user::User as AuthUser;
use bigneon_db::models::{DisplayUser, User};
use db::Connection;
use errors::*;
use helpers::application;
use models::register_request::RegisterRequest;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct SearchUserByEmail {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentUser {
    pub user: DisplayUser,
    pub roles: Vec<String>,
}

pub fn current_user(
    (connection, user): (Connection, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let user = User::find(user.id(), connection.get())?;
    let current_user = CurrentUser {
        roles: user.role.clone(),
        user: user.for_display(),
    };
    Ok(HttpResponse::Ok().json(&current_user))
}

pub fn show(
    (connection, parameters, user): (Connection, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::UserRead) {
        return application::unauthorized();
    }

    let user = User::find(parameters.id, connection.get())?;
    Ok(HttpResponse::Ok().json(&user.for_display()))
}

pub fn find_by_email(
    (connection, query, user): (Connection, Query<SearchUserByEmail>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::UserRead) {
        return application::unauthorized();
    }

    let user = User::find_by_email(&query.into_inner().email, connection.get())?;
    Ok(HttpResponse::Ok().json(&user.for_display()))
}

pub fn register(
    (connection, request): (Connection, Json<RegisterRequest>),
) -> Result<HttpResponse, BigNeonError> {
    User::create(
        &request.first_name,
        &request.last_name,
        &request.email,
        &request.phone,
        &request.password,
    ).commit(connection.get())?;
    Ok(HttpResponse::Ok().finish())
}
