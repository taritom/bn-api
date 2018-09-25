use actix_web::{HttpResponse, Json, Path};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::{AddVenueToOrganizationRequest, PathParameters};

pub fn index((connection, user): (Connection, Option<User>)) -> Result<HttpResponse, BigNeonError> {
    let venues = match user {
        Some(u) => Venue::all(Some(u.id()), connection.get())?,
        None => Venue::all(None, connection.get())?,
    };

    Ok(HttpResponse::Ok().json(&venues))
}

pub fn show(
    (connection, parameters): (Connection, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let venue = Venue::find(parameters.id, connection)?;

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
    let connection = connection.get();
    if !user.has_scope(Scopes::VenueWrite, None, connection)? {
        if new_venue.organization_id.is_none() {
            return application::unauthorized();
        } else if let Some(organization_id) = new_venue.organization_id {
            let organization = Organization::find(organization_id, connection)?;
            if !user.has_scope(Scopes::VenueWrite, Some(&organization), connection)? {
                return application::unauthorized();
            }
        }
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
    if !user.has_scope(Scopes::VenueWrite, None, connection)? {
        return application::unauthorized();
    }

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
    if !user.has_scope(Scopes::VenueWrite, None, connection)? {
        if !venue.is_private || venue.organization_id.is_none() {
            return application::unauthorized();
        } else if let Some(organization) = venue.organization(connection)? {
            if !user.has_scope(Scopes::VenueWrite, Some(&organization), connection)? {
                return application::unauthorized();
            }
        }
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
    if !user.has_scope(Scopes::OrgAdmin, None, connection)? {
        return application::unauthorized();
    }
    let venue = Venue::find(parameters.id, &*connection)?;
    let add_request = add_request.into_inner();
    if venue.organization_id.is_some() {
        Ok(HttpResponse::Conflict().json(json!({"error": "An error has occurred"})))
    } else {
        let venue = venue.add_to_organization(&add_request.organization_id, connection)?;
        Ok(HttpResponse::Created().json(&venue))
    }
}
