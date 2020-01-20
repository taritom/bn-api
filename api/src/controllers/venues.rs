use actix_web::{HttpResponse, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use extractors::*;
use models::{AddVenueToOrganizationRequest, PathParameters};

pub fn index(
    (connection, query_parameters, user): (Connection, Query<PagingParameters>, OptionalUser),
) -> Result<HttpResponse, BigNeonError> {
    //TODO implement proper paging on db
    let venues = match user.into_inner() {
        Some(u) => Venue::all(Some(&u.user), connection.get())?,
        None => Venue::all(None, connection.get())?,
    };

    Ok(HttpResponse::Ok().json(&Payload::from_data(
        venues,
        query_parameters.page(),
        query_parameters.limit(),
        None,
    )))
}

pub fn show((connection, parameters): (Connection, Path<PathParameters>)) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let venue = Venue::find(parameters.id, connection)?;

    Ok(HttpResponse::Ok().json(&venue))
}

pub fn show_from_organizations(
    (connection, organization_id, query_parameters, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        OptionalUser,
    ),
) -> Result<HttpResponse, BigNeonError> {
    //TODO implement proper paging on db
    let venues = match user.into_inner() {
        Some(u) => Venue::find_for_organization(Some(&u.user), organization_id.id, connection.get())?,
        None => Venue::find_for_organization(None, organization_id.id, connection.get())?,
    };

    Ok(HttpResponse::Ok().json(&Payload::from_data(
        venues,
        query_parameters.page(),
        query_parameters.limit(),
        None,
    )))
}

pub fn create((connection, new_venue, user): (Connection, Json<NewVenue>, User)) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();

    if let Some(organization_id) = new_venue.organization_id {
        let organization = Organization::find(organization_id, connection)?;
        user.requires_scope_for_organization(Scopes::VenueWrite, &organization, connection)?;
    } else {
        user.requires_scope(Scopes::VenueWrite)?;
    }

    let mut venue = new_venue.commit(connection)?;

    // New venues belonging to an organization start private
    if venue.organization_id.is_some() {
        venue = venue.set_privacy(true, connection)?;
    }

    Ok(HttpResponse::Created().json(&venue))
}

pub fn toggle_privacy(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    user.requires_scope(Scopes::VenueWrite)?;

    let venue = Venue::find(parameters.id, connection)?;
    let updated_venue = venue.set_privacy(!venue.is_private, connection)?;
    Ok(HttpResponse::Ok().json(updated_venue))
}

pub fn update(
    (connection, parameters, venue_parameters, user): (
        Connection,
        Path<PathParameters>,
        Json<VenueEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let venue = Venue::find(parameters.id, connection)?;
    if !venue.is_private || venue.organization_id.is_none() {
        user.requires_scope(Scopes::VenueWrite)?;
    } else {
        let organization = venue.organization(connection)?.unwrap();
        user.requires_scope_for_organization(Scopes::VenueWrite, &organization, connection)?;
    }

    let updated_venue = venue.update(venue_parameters.into_inner(), connection)?;
    Ok(HttpResponse::Ok().json(updated_venue))
}

pub fn add_to_organization(
    (connection, parameters, add_request, user): (
        Connection,
        Path<PathParameters>,
        Json<AddVenueToOrganizationRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    user.requires_scope(Scopes::OrgAdmin)?;
    let venue = Venue::find(parameters.id, &*connection)?;
    let add_request = add_request.into_inner();
    if venue.organization_id.is_some() {
        Ok(HttpResponse::Conflict().json(json!({"error": "An error has occurred"})))
    } else {
        let venue = venue.add_to_organization(&add_request.organization_id, connection)?;
        Ok(HttpResponse::Created().json(&venue))
    }
}
