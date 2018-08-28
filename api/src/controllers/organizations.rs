use actix_web::{HttpResponse, Json, Path, State};
use bigneon_db::models::{
    DisplayUser, NewOrganization, Organization, OrganizationEditableAttributes, OrganizationUser,
    User as DbUser,
};
use errors::*;

use auth::user::Scopes;
use auth::user::User;
use helpers::application;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateOwnerRequest {
    pub owner_user_id: Uuid,
}

pub fn index((state, user): (State<AppState>, User)) -> Result<HttpResponse, BigNeonError> {
    if user.has_scope(Scopes::OrgAdmin) {
        return index_for_all_orgs((state, user));
    }
    if !user.has_scope(Scopes::OrgRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organizations = Organization::all_linked_to_user(user.id(), &*connection)?;
    Ok(HttpResponse::Ok().json(&organizations))
}

pub fn index_for_all_orgs(
    (state, user): (State<AppState>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organizations = Organization::all(&*connection)?;
    Ok(HttpResponse::Ok().json(&organizations))
}

pub fn show(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization = Organization::find(parameters.id, &*connection)?;

    Ok(HttpResponse::Ok().json(&organization))
}

pub fn create(
    (state, new_organization, user): (State<AppState>, Json<NewOrganization>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();

    let organization = new_organization.commit(&*connection)?;
    Ok(HttpResponse::Created().json(&organization))
}

pub fn update(
    (state, parameters, organization_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<OrganizationEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization = Organization::find(parameters.id, &*connection)?;
    let updated_organization =
        organization.update(organization_parameters.into_inner(), &*connection)?;
    Ok(HttpResponse::Ok().json(&updated_organization))
}

pub fn update_owner(
    (state, parameters, json, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<UpdateOwnerRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization = Organization::find(parameters.id, &*connection)?;
    let updated_organization =
        organization.set_owner(json.into_inner().owner_user_id, &*connection)?;
    Ok(HttpResponse::Ok().json(&updated_organization))
}

pub fn remove_user(
    (state, parameters, user_id, user): (State<AppState>, Path<PathParameters>, Json<Uuid>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let organization = Organization::find(parameters.id, &*connection)?;

    let organization = organization.remove_user(&user_id.into_inner(), &*connection)?;
    Ok(HttpResponse::Ok().json(&organization))
}

pub fn list_organization_members(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgRead) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();

    let organization = Organization::find(parameters.id, &*connection)?;

    let members: Vec<DisplayUser> = organization
        .users(&*connection)?
        .iter()
        .map(|u| DisplayUser::from(u.clone()))
        .collect();

    Ok(HttpResponse::Ok().json(members))
}
