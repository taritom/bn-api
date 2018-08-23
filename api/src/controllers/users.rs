use actix_web::{HttpResponse, Json, Path, Query, State};
use auth::user::Scopes;
use auth::user::User as AuthUser;
use bigneon_db::models::{DisplayUser, User};
use errors::*;
use helpers::application;
use models::register_request::RegisterRequest;
use server::AppState;
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
    (state, user): (State<AppState>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let user = User::find(&user.id(), &*connection)?;
    let current_user = CurrentUser {
        roles: user.role.clone(),
        user: user.for_display(),
    };
    Ok(HttpResponse::Ok().json(&current_user))
}

pub fn show(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::UserRead) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    let user = User::find(&parameters.id, &*connection)?;
    Ok(HttpResponse::Ok().json(&user.for_display()))
}

pub fn find_by_email(
    (state, query, user): (State<AppState>, Query<SearchUserByEmail>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::UserRead) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    let user = User::find_by_email(&query.into_inner().email, &*connection)?;
    Ok(HttpResponse::Ok().json(&user.for_display()))
}

pub fn register(
    (state, request): (State<AppState>, Json<RegisterRequest>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();

    User::create(
        &request.first_name,
        &request.last_name,
        &request.email,
        &request.phone,
        &request.password,
    ).commit(&*connection)?;
    Ok(HttpResponse::Ok().finish())
}
