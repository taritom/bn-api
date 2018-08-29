use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{NewVenue, Venue, VenueEditableAttributes};
use errors::*;
use helpers::application;
use models::AddVenueToOrganizationRequest;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index((state, user): (State<AppState>, User)) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::VenueRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let venues = Venue::all(&*connection)?;
    Ok(HttpResponse::Ok().json(&venues))
}

pub fn show(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::VenueRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let venue = Venue::find(parameters.id, &*connection)?;
    Ok(HttpResponse::Ok().json(&venue))
}

pub fn show_from_organizations(
    (state, organization_id, user): (State<AppState>, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::VenueRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let venues = Venue::find_for_organization(organization_id.id, &*connection)?;
    Ok(HttpResponse::Ok().json(&venues))
}

pub fn create(
    (state, new_venue, user): (State<AppState>, Json<NewVenue>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::VenueWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let venue = new_venue.commit(&*connection)?;
    Ok(HttpResponse::Created().json(&venue))
}

pub fn update(
    (state, parameters, venue_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<VenueEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::VenueWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();

    let venue = Venue::find(parameters.id, &*connection)?;
    let updated_venue = venue.update(venue_parameters.into_inner(), &*connection)?;
    Ok(HttpResponse::Ok().json(updated_venue))
}

pub fn add_to_organization(
    (state, parameters, add_request, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<AddVenueToOrganizationRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::VenueWrite) || !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let add_request = add_request.into_inner();
    let venue = Venue::find(parameters.id, &*connection)?;
    let has_organization = venue.has_organization(add_request.organization_id, &*connection)?;

    if has_organization {
        Ok(HttpResponse::Conflict().json(json!({"error": "An error has occurred"})))
    } else {
        let organization_venue =
            venue.add_to_organization(&add_request.organization_id, &*connection)?;
        Ok(HttpResponse::Ok().json(&organization_venue))
    }
}
