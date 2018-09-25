use actix_web::{http::StatusCode, HttpResponse, Json, Path, Query};
use auth::user::User as AuthUser;
use bigneon_db::models::scopes;
use bigneon_db::models::*;
use bigneon_db::utils::errors::Optional;
use db::Connection;
use diesel::PgConnection;
use errors::*;
use helpers::application;
use models::{PathParameters, RegisterRequest, UserProfileAttributes};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct SearchUserByEmail {
    pub email: String,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentUser {
    pub user: DisplayUser,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
    pub organization_roles: HashMap<Uuid, Vec<String>>,
    pub organization_scopes: HashMap<Uuid, Vec<String>>,
}

pub fn current_user(
    (connection, user): (Connection, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let user = User::find(user.id(), connection)?;
    let current_user = current_user_from_user(&user, connection)?;
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
            let current_user = current_user_from_user(&updated_user, connection)?;
            Ok(HttpResponse::Ok().json(&current_user))
        }
        Err(e) => application::validation_error_response(e),
    }
}

pub fn show(
    (connection, parameters, auth_user): (Connection, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let user = User::find(parameters.id, connection)?;
    if !auth_user.user.can_read_user(&user, connection)? {
        return application::unauthorized();
    }

    Ok(HttpResponse::Ok().json(&user.for_display()))
}

pub fn list_organizations(
    (connection, parameters, auth_user): (Connection, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let user = User::find(parameters.id, connection)?;
    if !auth_user.user.can_read_user(&user, connection)? {
        return application::unauthorized();
    }

    let organization_links = Organization::all_org_names_linked_to_user(parameters.id, connection)?;
    Ok(HttpResponse::Ok().json(&organization_links))
}

pub fn find_by_email(
    (connection, query, auth_user): (Connection, Query<SearchUserByEmail>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let user = match User::find_by_email(&query.into_inner().email, connection).optional()? {
        Some(u) => u,
        None => return Ok(HttpResponse::new(StatusCode::NO_CONTENT)),
    };

    if !auth_user.user.can_read_user(&user, connection)? {
        return application::unauthorized();
    }

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

fn current_user_from_user(
    user: &User,
    connection: &PgConnection,
) -> Result<CurrentUser, BigNeonError> {
    let roles_by_organization = user.get_roles_by_organization(connection)?;
    let mut scopes_by_organization = HashMap::new();
    for (organization_id, roles) in &roles_by_organization {
        scopes_by_organization.insert(organization_id.clone(), scopes::get_scopes(roles.clone()));
    }

    Ok(CurrentUser {
        user: user.clone().for_display(),
        roles: user.role.clone(),
        scopes: user.get_global_scopes(),
        organization_roles: roles_by_organization,
        organization_scopes: scopes_by_organization,
    })
}
