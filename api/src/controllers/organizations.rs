use actix_web::{http::StatusCode, HttpResponse, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use chrono::NaiveDateTime;
use db::Connection;
use errors::*;
use extractors::*;
use models::PathParameters;
use models::WebPayload;
use uuid::Uuid;

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
    pub event_fee_in_cents: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub address: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub city: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub state: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub country: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub postal_code: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub phone: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub sendgrid_api_key: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub google_ga_key: Option<String>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    pub facebook_pixel_key: Option<String>,
}

pub fn index(
    (connection, query_parameters, user): (Connection, Query<PagingParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if user.requires_scope(Scopes::OrgAdmin).is_ok() {
        return index_for_all_orgs((connection, query_parameters, user));
    }

    //TODO remap query to use paging info
    let organizations = Organization::all_linked_to_user(user.id(), connection.get())?;

    Ok(HttpResponse::Ok().json(&Payload::from_data(
        organizations,
        query_parameters.page(),
        query_parameters.limit(),
    )))
}

pub fn index_for_all_orgs(
    (connection, query_parameters, user): (Connection, Query<PagingParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::OrgAdmin)?;
    let connection = connection.get();
    let organizations = Organization::all(connection)?;

    Ok(HttpResponse::Ok().json(&Payload::from_data(
        organizations,
        query_parameters.page(),
        query_parameters.limit(),
    )))
}

pub fn show(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    user.requires_scope_for_organization(Scopes::OrgRead, &organization, connection)?;

    Ok(HttpResponse::Ok().json(&organization))
}

pub fn create(
    (connection, new_organization, user): (Connection, Json<NewOrganizationRequest>, User),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::OrgAdmin)?;
    let connection = connection.get();

    let fee_schedule = FeeSchedule::create(
        format!("{} default fees", new_organization.name),
        vec![NewFeeScheduleRange {
            min_price_in_cents: 0,
            company_fee_in_cents: 0,
            client_fee_in_cents: 0,
        }],
    )
    .commit(connection)?;

    let new_organization_with_fee_schedule = NewOrganization {
        owner_user_id: new_organization.owner_user_id,
        name: new_organization.name.clone(),
        fee_schedule_id: fee_schedule.id,
        event_fee_in_cents: new_organization.event_fee_in_cents.clone(),
        address: new_organization.address.clone(),
        city: new_organization.city.clone(),
        state: new_organization.state.clone(),
        country: new_organization.country.clone(),
        postal_code: new_organization.postal_code.clone(),
        phone: new_organization.phone.clone(),
        sendgrid_api_key: new_organization.sendgrid_api_key.clone(),
        google_ga_key: new_organization.google_ga_key.clone(),
        facebook_pixel_key: new_organization.facebook_pixel_key.clone(),
    };

    let organization = new_organization_with_fee_schedule.commit(connection)?;

    Wallet::create_for_organization(organization.id, "Default".to_string(), connection)?;

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
    user.requires_scope_for_organization(Scopes::OrgWrite, &organization, connection)?;
    let organization_update = organization_parameters.into_inner();
    let updated_organization = organization.update(organization_update, connection)?;
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
    user.requires_scope(Scopes::OrgAdmin)?;
    let connection = connection.get();
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
    user.requires_scope_for_organization(Scopes::OrgWrite, &organization, connection)?;

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
    user.requires_scope_for_organization(Scopes::OrgWrite, &organization, connection)?;

    let mut new_artist = new_artist.into_inner();
    new_artist.organization_id = Some(parameters.id);

    let artist = new_artist.commit(connection)?;
    Ok(HttpResponse::Created().json(&artist))
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
    user.requires_scope_for_organization(Scopes::OrgWrite, &organization, connection)?;
    organization.add_user(add_request.user_id, None, connection)?;
    Ok(HttpResponse::Created().finish())
}

pub fn remove_user(
    (connection, parameters, user_id, user): (Connection, Path<PathParameters>, Json<Uuid>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    user.requires_scope_for_organization(Scopes::OrgWrite, &organization, connection)?;

    let organization = organization.remove_user(user_id.into_inner(), connection)?;
    Ok(HttpResponse::Ok().json(&organization))
}

pub fn list_organization_members(
    (connection, path_parameters, query_parameters, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    //TODO refactor Organization::find to use limits as in PagingParameters
    let organization = Organization::find(path_parameters.id, connection)?;
    user.requires_scope_for_organization(Scopes::OrgRead, &organization, connection)?;

    let mut members: Vec<DisplayUser> = organization
        .users(connection)?
        .iter()
        .map(|u| DisplayUser::from(u.clone()))
        .collect();
    members[0].is_org_owner = true;
    let payload = Payload::from_data(members, query_parameters.page(), query_parameters.limit());
    Ok(HttpResponse::Ok().json(payload))
}

pub fn show_fee_schedule(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(parameters.id, connection)?;
    user.requires_scope_for_organization(Scopes::OrgWrite, &organization, connection)?;

    //This is an OrgOwner / Admin only call so we need to show the breakdown
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
    user.requires_scope(Scopes::OrgAdmin)?;
    let connection = connection.get();

    let fee_schedule = json.into_inner().commit(connection)?;
    let fee_schedule_ranges = fee_schedule.ranges(connection)?;

    Organization::find(parameters.id, connection)?.add_fee_schedule(&fee_schedule, connection)?;

    Ok(HttpResponse::Created().json(FeeScheduleWithRanges {
        id: fee_schedule.id,
        name: fee_schedule.name,
        version: fee_schedule.version,
        created_at: fee_schedule.created_at,
        ranges: fee_schedule_ranges,
    }))
}

pub fn search_fans(
    (connection, path, query, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        User,
    ),
) -> Result<WebPayload<DisplayFan>, BigNeonError> {
    let connection = connection.get();
    let org = Organization::find(path.id, connection)?;
    user.requires_scope_for_organization(Scopes::OrgReadFans, &org, &connection)?;
    let payload = org.search_fans(
        query.get_tag("query"),
        query.page(),
        query.limit(),
        query
            .sort
            .as_ref()
            .map(|s| s.parse().unwrap_or(FanSortField::LastOrder))
            .unwrap_or(FanSortField::LastOrder),
        query.dir.unwrap_or(SortingDir::Desc),
        connection,
    )?;
    Ok(WebPayload::new(StatusCode::OK, payload))
}
