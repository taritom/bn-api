use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{NewVenue, Venue};
use errors::database_error::ConvertToWebError;
use helpers::application;
use models::AddVenueToOrganizationRequest;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index((state, user): (State<AppState>, User)) -> HttpResponse {
    if !user.has_scope(Scopes::VenueRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let venue_response = Venue::all(&*connection);
    match venue_response {
        Ok(venues) => HttpResponse::Ok().json(&venues),
        Err(_e) => {
            HttpResponse::InternalServerError().json(json!({"error": "An error has occurred"}))
        }
    }
}

pub fn show(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> HttpResponse {
    if !user.has_scope(Scopes::VenueRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let venue_response = Venue::find(&parameters.id, &*connection);

    match venue_response {
        Ok(venue) => HttpResponse::Ok().json(&venue),
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Venue not found"})),
    }
}

pub fn show_from_organizations(
    (state, organization_id, user): (State<AppState>, Path<PathParameters>, User),
) -> HttpResponse {
    if !user.has_scope(Scopes::VenueRead) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let venue_response = Venue::find_for_organization(organization_id.id, &*connection);
    match venue_response {
        Ok(venues) => HttpResponse::Ok().json(&venues),
        Err(e) => HttpResponse::from_error(e.create_http_error()),
    }
}

pub fn create((state, new_venue, user): (State<AppState>, Json<NewVenue>, User)) -> HttpResponse {
    if !user.has_scope(Scopes::VenueWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let venue_response = new_venue.commit(&*connection);

    match venue_response {
        Ok(venue) => HttpResponse::Created().json(&venue),
        Err(_e) => HttpResponse::BadRequest().json(json!({"error": "An error has occurred"})),
    }
}

pub fn update(data: (State<AppState>, Path<PathParameters>, Json<Venue>, User)) -> HttpResponse {
    let (state, parameters, venue_parameters, user) = data;
    if !user.has_scope(Scopes::VenueWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();

    let updated_venue: Venue = venue_parameters.into_inner();
    let venue_response = Venue::find(&parameters.id, &*connection);
    match venue_response {
        Ok(_venue) => {
            let venue_update_response = updated_venue.update(&*connection);
            match venue_update_response {
                Ok(updated_venue) => HttpResponse::Ok().json(&updated_venue),
                Err(_e) => {
                    HttpResponse::BadRequest().json(json!({"error": "An error has occurred"}))
                }
            }
        }
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Venue not found"})),
    }
}

pub fn add_to_organization(
    (state, parameters, add_request, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<AddVenueToOrganizationRequest>,
        User,
    ),
) -> HttpResponse {
    if !user.has_scope(Scopes::VenueWrite) || !user.has_scope(Scopes::OrgWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let add_request = add_request.into_inner();
    match Venue::find(&parameters.id, &*connection) {
        Ok(venue) => match venue.has_organization(add_request.organization_id, &*connection) {
            Ok(has_organization) => {
                if has_organization {
                    HttpResponse::Conflict().json(json!({"error": "An error has occurred"}))
                } else {
                    let venue_update_response =
                        venue.add_to_organization(&add_request.organization_id, &*connection);
                    match venue_update_response {
                        Ok(organization_venue) => HttpResponse::Ok().json(&organization_venue),
                        Err(e) => {
                            HttpResponse::from_error(ConvertToWebError::create_http_error(&e))
                        }
                    }
                }
            }
            Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
        },
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Venue not found"})),
    }
}
