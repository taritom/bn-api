use actix_web::{HttpResponse, Json, Path, State};
use auth::user::User;
use bigneon_db::models::{NewVenue, Roles, Venue};
use helpers::application;
use models::AddVenueToOrganizationRequest;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(state: State<AppState>) -> HttpResponse {
    let connection = state.database.get_connection();
    let venue_response = Venue::all(&*connection);
    match venue_response {
        Ok(venues) => HttpResponse::Ok().json(&venues),
        Err(_e) => {
            HttpResponse::InternalServerError().json(json!({"error": "An error has occurred"}))
        }
    }
}

pub fn show((state, parameters): (State<AppState>, Path<PathParameters>)) -> HttpResponse {
    //    let user = User::new("username", "roles");
    //    user.requires_role("Guest")?;
    let connection = state.database.get_connection();
    let venue_response = Venue::find(&parameters.id, &*connection);

    match venue_response {
        Ok(venue) => HttpResponse::Ok().json(&venue),
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Venue not found"})),
    }
}

pub fn show_from_organizations(data: (State<AppState>, Path<PathParameters>)) -> HttpResponse {
    let (state, organization_id) = data;
    //    let user = User::new("username", "roles");
    //    user.requires_role("Guest")?;
    let connection = state.database.get_connection();
    let venue_response = Venue::find_for_organization(organization_id.id, &*connection);
    match venue_response {
        Ok(venues) => HttpResponse::Ok().json(&venues),
        Err(_e) => {
            HttpResponse::InternalServerError().json(json!({"error": "An error has occurred"}))
        }
    }
}

pub fn create((state, new_venue, user): (State<AppState>, Json<NewVenue>, User)) -> HttpResponse {
    if !user.is_in_role(Roles::Admin) {
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
    if !user.is_in_role(Roles::Admin) {
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
    data: (
        State<AppState>,
        Path<PathParameters>,
        Json<AddVenueToOrganizationRequest>,
        User,
    ),
) -> HttpResponse {
    let (state, parameters, add_request, user) = data;
    let connection = state.database.get_connection();
    let add_request = add_request.into_inner();
    if !user.is_in_role(Roles::Admin) {
        return application::unauthorized();
    }
    let venue_response = Venue::find(&parameters.id, &*connection);
    match venue_response {
        Ok(venue) => {
            let venue_update_response =
                venue.add_to_organization(&add_request.organization_id, &*connection);
            match venue_update_response {
                Ok(organization_venue) => HttpResponse::Ok().json(&organization_venue),
                Err(_e) => {
                    HttpResponse::BadRequest().json(json!({"error": "An error has occurred"}))
                }
            }
        }
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Venue not found"})),
    }
}
