use actix_web::{http::StatusCode, HttpRequest, HttpResponse, Json, Path, Query, State};
use auth::user::User as AuthUser;
use bigneon_db::models::*;
use bigneon_db::utils::errors::Optional;
use communications::mailers;
use controllers::auth;
use controllers::auth::LoginRequest;
use db::Connection;
use diesel::PgConnection;
use errors::*;
use helpers::application;
use models::*;
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

#[derive(Deserialize, Clone)]
pub struct InputPushNotificationTokens {
    pub token_source: String,
    pub token: String,
}

pub fn current_user(
    (connection, auth_user): (Connection, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let current_user = current_user_from_user(&auth_user.user, connection)?;
    Ok(HttpResponse::Ok().json(&current_user))
}

pub fn profile(
    (connection, path, auth_user, request): (
        Connection,
        Path<OrganizationFanPathParameters>,
        AuthUser,
        HttpRequest<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(path.id, connection)?;
    auth_user.requires_scope_for_organization(Scopes::OrgReadFans, &organization, &connection)?;

    let user = User::find(path.user_id, connection)?;

    // Confirm organization has specified user as a fan
    if !organization.has_fan(&user, connection)? {
        return application::unauthorized(&request, Some(auth_user));
    }

    Ok(HttpResponse::Ok().json(&user.get_profile_for_organization(&organization, connection)?))
}

pub fn history(
    (connection, path, query, auth_user, request): (
        Connection,
        Path<OrganizationFanPathParameters>,
        Query<PagingParameters>,
        AuthUser,
        HttpRequest<AppState>,
    ),
) -> Result<WebPayload<HistoryItem>, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(path.id, connection)?;
    auth_user.requires_scope_for_organization(Scopes::OrgReadFans, &organization, &connection)?;

    let user = User::find(path.user_id, connection)?;

    // Confirm organization has specified user as a fan
    if !organization.has_fan(&user, connection)? {
        return application::unauthorized(&request, Some(auth_user));
    }

    let payload = user.get_history_for_organization(
        &organization,
        query.page(),
        query.limit(),
        query.dir.unwrap_or(SortingDir::Desc),
        connection,
    )?;

    Ok(WebPayload::new(StatusCode::OK, payload))
}

pub fn update_current_user(
    (connection, user_parameters, auth_user): (Connection, Json<UserProfileAttributes>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();

    let updated_user = auth_user
        .user
        .update(&user_parameters.into_inner().into(), connection)?;
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

pub fn show_push_notification_tokens_for_user_id(
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

    let push_notification_tokens: Vec<DisplayPushNotificationToken> =
        PushNotificationToken::find_by_user_id(parameters.id, connection)?
            .iter()
            .map(|t| DisplayPushNotificationToken::from(t.clone()))
            .collect();

    Ok(HttpResponse::Ok().json(&push_notification_tokens))
}

pub fn show_push_notification_tokens(
    (connection, auth_user): (Connection, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();

    let push_notification_tokens: Vec<DisplayPushNotificationToken> =
        PushNotificationToken::find_by_user_id(auth_user.user.id, connection)?
            .iter()
            .map(|t| DisplayPushNotificationToken::from(t.clone()))
            .collect();

    Ok(HttpResponse::Ok().json(&push_notification_tokens))
}

pub fn add_push_notification_token(
    (connection, add_request, auth_user): (Connection, Json<InputPushNotificationTokens>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();

    let add_push_notification_token_request = add_request.into_inner();
    PushNotificationToken::create(
        auth_user.user.id,
        add_push_notification_token_request.token_source.clone(),
        add_push_notification_token_request.token.clone(),
    )
    .commit(connection)?;

    Ok(HttpResponse::Ok().finish())
}

pub fn remove_push_notification_token(
    (connection, parameters, auth_user): (Connection, Path<PathParameters>, AuthUser),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();

    PushNotificationToken::remove(auth_user.user.id, parameters.id, connection)?;

    Ok(HttpResponse::Ok().finish())
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
        mailers::user::user_registered(first_name, email, &state.config, connection.get())?;
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
    let token_response = auth::token((http_request, connection.clone(), json))?;

    if let (Some(first_name), Some(email)) = (new_user.first_name, new_user.email) {
        mailers::user::user_registered(first_name, email, &state.config, connection.get())?;
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
