use actix_web::{HttpResponse, Json, Path};
use auth::user::User;
use bigneon_db::models::*;
use chrono::NaiveDateTime;
use db::Connection;
use errors::*;
use helpers::application;
use uuid::Uuid;
use validator::Validate;

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

#[derive(Serialize, Deserialize)]
pub struct FeeScheduleWithRanges {
    pub id: Uuid,
    pub name: String,
    pub version: i16,
    pub created_at: NaiveDateTime,
    pub ranges: Vec<FeeScheduleRange>,
}

#[derive(Serialize, Deserialize)]
pub struct NewOrganizationRequest {
    pub owner_user_id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub phone: Option<String>,
}

pub fn index((connection, user): (Connection, User)) -> Result<HttpResponse, BigNeonError> {
    if user.has_scope(Scopes::OrgAdmin, None, connection.get())? {
        return index_for_all_orgs((connection, user));
    }
    let organizations = Organization::all_linked_to_user(user.id(), connection.get())?;
    Ok(HttpResponse::Ok().json(&organizations))
}

pub fn index_for_all_orgs(
    (connection, user): (Connection, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::OrgAdmin, None, connection)? {
        return application::unauthorized();
    }
    let organizations = Organization::all(connection)?;
    Ok(HttpResponse::Ok().json(&organizations))
}

pub fn show(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    if !user.has_scope(Scopes::OrgRead, Some(&organization), connection)? {
        return application::unauthorized();
    }

    Ok(HttpResponse::Ok().json(&organization))
}

pub fn create(
    (connection, new_organization, user): (Connection, Json<NewOrganizationRequest>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::OrgAdmin, None, connection)? {
        return application::unauthorized();
    }

    let fee_schedule = FeeSchedule::create(
        format!("{} default fees", new_organization.name).into(),
        vec![NewFeeScheduleRange {
            min_price: 0,
            fee_in_cents: 0,
        }],
    ).commit(connection)?;

    let new_organization_with_fee_schedule = NewOrganization {
        owner_user_id: new_organization.owner_user_id.clone(),
        name: new_organization.name.clone(),
        fee_schedule_id: fee_schedule.id,
        address: new_organization.address.clone(),
        city: new_organization.city.clone(),
        state: new_organization.state.clone(),
        country: new_organization.country.clone(),
        postal_code: new_organization.postal_code.clone(),
        phone: new_organization.phone.clone(),
    };

    let organization = new_organization_with_fee_schedule.commit(connection)?;
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
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    if !user.has_scope(Scopes::OrgWrite, Some(&organization), connection)? {
        return application::unauthorized();
    }

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
    let connection = connection.get();
    if !user.has_scope(Scopes::OrgAdmin, None, connection)? {
        return application::unauthorized();
    }
    let organization = Organization::find(parameters.id, connection)?;
    let updated_organization =
        organization.set_owner(json.into_inner().owner_user_id, connection)?;
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
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    if !user.has_scope(Scopes::OrgWrite, Some(&organization), connection)? {
        return application::unauthorized();
    }

    let mut new_venue = new_venue.into_inner();
    new_venue.organization_id = Some(parameters.id);
    let venue = new_venue.commit(connection)?;
    Ok(HttpResponse::Created().json(&venue))
}

pub fn add_artist(
    (connection, parameters, new_artist, user): (
        Connection,
        Path<PathParameters>,
        Json<NewArtist>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    if !user.has_scope(Scopes::OrgWrite, Some(&organization), connection)? {
        return application::unauthorized();
    }

    let mut new_artist = new_artist.into_inner();
    new_artist.organization_id = Some(parameters.id);

    match new_artist.validate() {
        Ok(_) => {
            let artist = new_artist.commit(connection)?;
            Ok(HttpResponse::Created().json(&artist))
        }
        Err(e) => application::validation_error_response(e),
    }
}

pub fn add_user(
    (connection, path, add_request, user): (
        Connection,
        Path<PathParameters>,
        Json<AddUserRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(path.id, connection)?;
    if !user.has_scope(Scopes::OrgWrite, Some(&organization), connection)? {
        return application::unauthorized();
    }
    organization.add_user(add_request.user_id, connection)?;
    Ok(HttpResponse::Created().finish())
}

pub fn remove_user(
    (connection, parameters, user_id, user): (Connection, Path<PathParameters>, Json<Uuid>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    if !user.has_scope(Scopes::OrgWrite, Some(&organization), connection)? {
        return application::unauthorized();
    }

    let organization = organization.remove_user(user_id.into_inner(), connection)?;
    Ok(HttpResponse::Ok().json(&organization))
}

pub fn list_organization_members(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    if !user.has_scope(Scopes::OrgRead, Some(&organization), connection)? {
        return application::unauthorized();
    }

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
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    if !user.has_scope(Scopes::OrgWrite, Some(&organization), connection)? {
        return application::unauthorized();
    }

    let fee_schedule = FeeSchedule::find(organization.fee_schedule_id, connection)?;
    let fee_schedule_ranges = fee_schedule.ranges(connection)?;

    Ok(HttpResponse::Ok().json(FeeScheduleWithRanges {
        id: fee_schedule.id,
        name: fee_schedule.name,
        version: fee_schedule.version,
        created_at: fee_schedule.created_at,
        ranges: fee_schedule_ranges,
    }))
}

pub fn add_fee_schedule(
    (connection, parameters, json, user): (
        Connection,
        Path<PathParameters>,
        Json<NewFeeSchedule>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    if !user.has_scope(Scopes::OrgAdmin, None, connection)? {
        return application::unauthorized();
    }

    let fee_schedule = json.into_inner().commit(connection)?;
    let fee_schedule_ranges = fee_schedule.ranges(connection)?;

    let update_fee_schedule_id = OrganizationEditableAttributes {
        name: None,
        address: None,
        city: None,
        state: None,
        country: None,
        postal_code: None,
        phone: None,
        fee_schedule_id: Some(fee_schedule.id),
    };

    let organization = Organization::find(parameters.id, connection)?
        .update(update_fee_schedule_id, connection)?;

    Ok(HttpResponse::Created().json(FeeScheduleWithRanges {
        id: fee_schedule.id,
        name: fee_schedule.name,
        version: fee_schedule.version,
        created_at: fee_schedule.created_at,
        ranges: fee_schedule_ranges,
    }))
}
