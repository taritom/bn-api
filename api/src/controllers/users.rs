use actix_web::{http::StatusCode, HttpRequest, HttpResponse, Json, Path, Query, State};
use auth::user::User as AuthUser;
use bigneon_db::models::*;
use bigneon_db::utils::errors::Optional;
use controllers::auth;
use controllers::auth::LoginRequest;
use db::Connection;
use diesel::PgConnection;
use errors::*;
use helpers::application;
use mail::mailers;
use models::{PathParameters, RegisterRequest, UserProfileAttributes};
use server::AppState;
use std::collections::HashMap;
use std::str;
use uuid::Uuid;

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

    let updated_user = user.update(&user_parameters.into_inner().into(), connection)?;
    let current_user = current_user_from_user(&updated_user, connection)?;
    Ok(HttpResponse::Ok().json(&current_user))
}

pub fn show(
    (connection, parameters, auth_user, request): (
        Connection,
        Path<PathParameters>,
        AuthUser,
        HttpRequest<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let user = User::find(parameters.id, connection)?;
    if !auth_user.user.can_read_user(&user, connection)? {
        return application::unauthorized(&request, Some(auth_user));
    }

    Ok(HttpResponse::Ok().json(&user.for_display()?))
}

pub fn list_organizations(
    (connection, parameters, query_parameters, auth_user, request): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        AuthUser,
        HttpRequest<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let user = User::find(parameters.id, connection)?;
    if !auth_user.user.can_read_user(&user, connection)? {
        return application::unauthorized(&request, Some(auth_user));
    }
    //TODO implement proper paging on db.
    let organization_links = Organization::all_org_names_linked_to_user(parameters.id, connection)?;

    Ok(HttpResponse::Ok().json(&Payload::from_data(
        organization_links,
        query_parameters.page(),
        query_parameters.limit(),
    )))
}

pub fn find_by_email(
    (connection, query, auth_user, request): (
        Connection,
        Query<SearchUserByEmail>,
        AuthUser,
        HttpRequest<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let user = match User::find_by_email(&query.into_inner().email, connection).optional()? {
        Some(u) => u,
        None => return Ok(HttpResponse::new(StatusCode::NO_CONTENT)),
    };

    if !auth_user.user.can_read_user(&user, connection)? {
        return application::unauthorized(&request, Some(auth_user));
    }

    Ok(HttpResponse::Ok().json(&user.for_display()?))
}

pub fn register(
    (connection, parameters, state): (Connection, Json<RegisterRequest>, State<AppState>),
) -> Result<HttpResponse, BigNeonError> {
    let new_user: NewUser = parameters.into_inner().into();
    new_user.commit(connection.get())?;

    if let (Some(first_name), Some(email)) = (new_user.first_name, new_user.email) {
        mailers::user::user_registered(&first_name, &email, &state.config)?;
    }

    Ok(HttpResponse::Created().finish())
}

pub fn register_and_login(
    (http_request, connection, parameters, state): (
        HttpRequest<AppState>,
        Connection,
        Json<RegisterRequest>,
        State<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let email = parameters.email.clone();
    let password = parameters.password.clone();
    let new_user: NewUser = parameters.into_inner().into();
    new_user.commit(connection.get())?;
    let json = Json(LoginRequest::new(&email, &password));
    let token_response = auth::token((http_request, connection, json))?;

    if let (Some(first_name), Some(email)) = (new_user.first_name, new_user.email) {
        mailers::user::user_registered(&first_name, &email, &state.config)?;
    }

    Ok(HttpResponse::Created().json(token_response))
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
        user: user.clone().for_display()?,
        roles: user.role.clone(),
        scopes: user.get_global_scopes(),
        organization_roles: roles_by_organization,
        organization_scopes: scopes_by_organization,
    })
}
