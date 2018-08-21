use actix_web::{HttpResponse, Json, Path, State};
use bigneon_db::models::{NewOrganization, Organization, OrganizationEditableAttributes};
use errors::database_error::ConvertToWebError;

use auth::user::Scopes;
use auth::user::User;
use helpers::application;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index((state, user): (State<AppState>, User)) -> HttpResponse {
    if user.has_scope(Scopes::OrgAdmin) {
        return index_for_all_orgs((state, user));
    }
    if !user.has_scope(Scopes::OrgRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization_response = Organization::all_linked_to_user(user.id(), &*connection);
    match organization_response {
        Ok(organizations) => HttpResponse::Ok().json(&organizations),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn index_for_all_orgs((state, user): (State<AppState>, User)) -> HttpResponse {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization_response = Organization::all(&*connection);
    match organization_response {
        Ok(organizations) => HttpResponse::Ok().json(&organizations),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> HttpResponse {
    if !user.has_scope(Scopes::OrgRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization_response = Organization::find(parameters.id, &*connection);

    match organization_response {
        Ok(organization) => HttpResponse::Ok().json(&organization),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn create(
    (state, new_organization, user): (State<AppState>, Json<NewOrganization>, User),
) -> HttpResponse {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization_response = new_organization.commit(&*connection);
    match organization_response {
        Ok(organization) => HttpResponse::Created().json(&organization),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn update(
    (state, parameters, organization_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<OrganizationEditableAttributes>,
        User,
    ),
) -> HttpResponse {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    match Organization::find(parameters.id, &*connection) {
        Ok(organization) => {
            match organization.update(organization_parameters.into_inner(), &*connection) {
                Ok(updated_organization) => HttpResponse::Ok().json(&updated_organization),
                Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
            }
        }
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn update_owner(
    (state, parameters, json, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<UpdateOwnerRequest>,
        User,
    ),
) -> HttpResponse {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    match Organization::find(parameters.id, &*connection) {
        Ok(organization) => {
            match organization.set_owner(json.into_inner().owner_user_id, &*connection) {
                Ok(updated_organization) => HttpResponse::Ok().json(&updated_organization),
                Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
            }
        }
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn remove_user(
    (state, parameters, user_id, user): (State<AppState>, Path<PathParameters>, Json<Uuid>, User),
) -> HttpResponse {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization = Organization::find(parameters.id, &*connection).unwrap();
    let organization_response = organization.remove_user(&user_id.into_inner(), &*connection);
    match organization_response {
        Ok(organization) => HttpResponse::Ok().json(&organization),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateOwnerRequest {
    pub owner_user_id: Uuid,
}
