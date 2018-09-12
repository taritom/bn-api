use actix_web::{HttpResponse, Json, Path, Query};
use auth::user::Scopes;
use auth::user::User as AuthUser;
use bigneon_db::models::{DisplayUser, NewUser, Organization, User};
use db::Connection;
use errors::*;
use helpers::application;
use models::{RegisterRequest, UserProfileAttributes};
use uuid::Uuid;
use validator::Validate;

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

pub fn update_current_user(
    (connection, user_parameters, user): (Connection, Json<UserProfileAttributes>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let user = User::find(user.id(), connection)?;
    match user_parameters.validate() {
        Ok(_) => {
            let updated_user = user.update(&user_parameters.into_inner().into(), connection)?;
            let current_user = CurrentUser {
                roles: updated_user.role.clone(),
                user: updated_user.for_display(),
            };
            Ok(HttpResponse::Ok().json(&current_user))
        }
        Err(e) => application::validation_error_response(e),
    }
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

pub fn list_organizations(
    (connection, parameters, user): (Connection, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::UserRead) {
        return application::unauthorized();
    }
    let organization_links =
        Organization::all_org_names_linked_to_user(parameters.id, connection.get())?;
    Ok(HttpResponse::Ok().json(&organization_links))
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
    (connection, parameters): (Connection, Json<RegisterRequest>),
) -> Result<HttpResponse, BigNeonError> {
    let new_user: NewUser = parameters.into_inner().into();
    match new_user.validate() {
        Ok(_) => {
            new_user.commit(connection.get())?;
            Ok(HttpResponse::Created().finish())
        }
        Err(e) => application::validation_error_response(e),
    }
}
