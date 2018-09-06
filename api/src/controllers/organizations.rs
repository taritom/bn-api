use actix_web::{HttpResponse, Json, Path};
use auth::user::{Scopes, User};
use bigneon_db::models::{
    DisplayUser, FeeSchedule, FeeScheduleRange, NewOrganization, NewVenue, Organization,
    OrganizationEditableAttributes,
};
use chrono::NaiveDateTime;
use db::Connection;
use errors::*;
use helpers::application;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateOwnerRequest {
    pub owner_user_id: Uuid,
}

#[derive(Deserialize)]
pub struct AddUserRequest {
    pub user_id: Uuid,
}

pub fn index((connection, user): (Connection, User)) -> Result<HttpResponse, BigNeonError> {
    if user.has_scope(Scopes::OrgAdmin) {
        return index_for_all_orgs((connection, user));
    }
    if !user.has_scope(Scopes::OrgRead) {
        return application::unauthorized();
    }
    let organizations = Organization::all_linked_to_user(user.id(), connection.get())?;
    Ok(HttpResponse::Ok().json(&organizations))
}

pub fn index_for_all_orgs(
    (connection, user): (Connection, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }
    let organizations = Organization::all(connection.get())?;
    Ok(HttpResponse::Ok().json(&organizations))
}

pub fn show(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgRead) {
        return application::unauthorized();
    }
    let organization = Organization::find(parameters.id, connection.get())?;

    Ok(HttpResponse::Ok().json(&organization))
}

pub fn create(
    (connection, new_organization, user): (Connection, Json<NewOrganization>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }

    let organization = new_organization.commit(connection.get())?;
    Ok(HttpResponse::Created().json(&organization))
}

pub fn update(
    (connection, parameters, organization_parameters, user): (
        Connection,
        Path<PathParameters>,
        Json<OrganizationEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    let updated_organization =
        organization.update(organization_parameters.into_inner(), connection)?;
    Ok(HttpResponse::Ok().json(&updated_organization))
}

pub fn update_owner(
    (connection, parameters, json, user): (
        Connection,
        Path<PathParameters>,
        Json<UpdateOwnerRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgAdmin) {
        return application::unauthorized();
    }
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    let updated_organization = organization.set_owner(json.into_inner().owner_user_id, connection)?;
    Ok(HttpResponse::Ok().json(&updated_organization))
}

pub fn add_venue(
    (connection, parameters, new_venue, user): (
        Connection,
        Path<PathParameters>,
        Json<NewVenue>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let mut new_venue = new_venue.into_inner();
    new_venue.organization_id = Some(parameters.id);
    let venue = new_venue.commit(connection.get())?;
    Ok(HttpResponse::Created().json(&venue))
}

pub fn add_user(
    (connection, path, add_request, user): (
        Connection,
        Path<PathParameters>,
        Json<AddUserRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = connection.get();
    let org = Organization::find(path.id, connection)?;
    org.add_user(add_request.user_id, connection)?;
    Ok(HttpResponse::Ok().finish())
}

pub fn remove_user(
    (connection, parameters, user_id, user): (Connection, Path<PathParameters>, Json<Uuid>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;

    let organization = organization.remove_user(user_id.into_inner(), connection)?;
    Ok(HttpResponse::Ok().json(&organization))
}

pub fn list_organization_members(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgRead) {
        return application::unauthorized();
    }

    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;

    let mut members: Vec<DisplayUser> = organization
        .users(connection)?
        .iter()
        .map(|u| DisplayUser::from(u.clone()))
        .collect();

    #[derive(Serialize)]
    struct OrgOwnerMembers {
        organization_owner: DisplayUser,
        organization_members: Vec<DisplayUser>,
    }

    let org_owner_members = OrgOwnerMembers {
        organization_owner: members.remove(0),
        organization_members: members,
    };

    Ok(HttpResponse::Ok().json(org_owner_members))
}

pub fn show_fee_schedule(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = connection.get();

    let organization = Organization::find(parameters.id, connection)?;
    if organization.fee_schedule_id.is_none() {
        return Ok(HttpResponse::NotFound().finish());
    }
    let fee_schedule = FeeSchedule::find(organization.fee_schedule_id.unwrap(), connection)?;
    let fee_schedule_ranges = fee_schedule.ranges(connection)?;

    #[derive(Serialize)]
    struct FeeScheduleWithRanges {
        id: Uuid,
        name: String,
        created_at: NaiveDateTime,
        ranges: Vec<FeeScheduleRange>,
    }

    Ok(HttpResponse::Ok().json(FeeScheduleWithRanges {
        id: fee_schedule.id,
        name: fee_schedule.name,
        created_at: fee_schedule.created_at,
        ranges: fee_schedule_ranges,
    }))
}
