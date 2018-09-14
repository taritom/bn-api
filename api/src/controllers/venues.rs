use actix_web::{HttpResponse, Json, Path};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{NewVenue, Organization, Venue, VenueEditableAttributes};
use db::Connection;
use errors::*;
use helpers::application;
use models::AddVenueToOrganizationRequest;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index((connection, user): (Connection, Option<User>)) -> Result<HttpResponse, BigNeonError> {
    let venues = match user {
        Some(u) => Venue::all(Some(u.id()), connection.get())?,
        None => Venue::all(None, connection.get())?,
    };

    Ok(HttpResponse::Ok().json(&venues))
}

pub fn show(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::VenueRead) {
        return application::unauthorized();
    }
    let venue = Venue::find(parameters.id, connection.get())?;
    Ok(HttpResponse::Ok().json(&venue))
}

pub fn show_from_organizations(
    (connection, organization_id, user): (Connection, Path<PathParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    let venues = match user {
        Some(u) => {
            Venue::find_for_organization(Some(u.id()), organization_id.id, connection.get())?
        }
        None => Venue::find_for_organization(None, organization_id.id, connection.get())?,
    };
    Ok(HttpResponse::Ok().json(&venues))
}

pub fn create(
    (connection, new_venue, user): (Connection, Json<NewVenue>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::VenueWrite) {
        return application::unauthorized();
    }
    let venue = new_venue.commit(connection.get())?;
    Ok(HttpResponse::Created().json(&venue))
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

    if !user.has_scope(Scopes::VenueWrite) && !(user.has_scope(Scopes::OrgWrite)
        && venue.organization_id.is_some()
        && Organization::find(venue.organization_id.unwrap(), connection)?
            .is_member(&user.user, connection)?)
    {
        return application::unauthorized();
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
    if !user.has_scope(Scopes::VenueWrite) || !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = connection.get();
    let add_request = add_request.into_inner();
    let venue = Venue::find(parameters.id, &*connection)?;

    if venue.organization_id.is_some() {
        Ok(HttpResponse::Conflict().json(json!({"error": "An error has occurred"})))
    } else {
        let venue = venue.add_to_organization(&add_request.organization_id, connection)?;
        Ok(HttpResponse::Created().json(&venue))
    }
}
