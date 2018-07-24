use actix_web::{HttpResponse, Json, Path, State};
use bigneon_db::models::{NewVenue, Venue};
use helpers::application;
use models::user::User;
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

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> HttpResponse {
    let (state, parameters) = data;
    let connection = state.database.get_connection();
    let venue_response = Venue::find(&parameters.id, &*connection);

    match venue_response {
        Ok(venue) => HttpResponse::Ok().json(&venue),
        Err(_e) => HttpResponse::NotFound().json(json!({"error": "Venue not found"})),
    }
}

pub fn show_from_organizations(data: (State<AppState>, Json<Uuid>)) -> HttpResponse {
    let (state, organization_id) = data;
    let connection = state.database.get_connection();
    let venue_response =
        Venue::find_all_for_organization(&organization_id.into_inner(), &*connection);
    match venue_response {
        Ok(venues) => HttpResponse::Ok().json(&venues),
        Err(_e) => {
            HttpResponse::InternalServerError().json(json!({"error": "An error has occurred"}))
        }
    }
}

pub fn create(data: (State<AppState>, Json<NewVenue>)) -> HttpResponse {
    let (state, new_venue) = data;
    let connection = state.database.get_connection();
    let venue_response = new_venue.commit(&*connection);

    let user = User::new("username", "roles");
    if user.requires_role("Admin").is_err() {
        return application::unauthorized();
    }
    match venue_response {
        Ok(venue) => HttpResponse::Created().json(&venue),
        Err(_e) => HttpResponse::BadRequest().json(json!({"error": "An error has occurred"})),
    }
}

pub fn update(data: (State<AppState>, Path<PathParameters>, Json<Venue>)) -> HttpResponse {
    let (state, parameters, venue_parameters) = data;
    let connection = state.database.get_connection();
    let user = User::new("username", "roles");
    if user.requires_role("Admin").is_err() {
        return application::unauthorized();
    }

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
    data: (State<AppState>, Path<PathParameters>, Json<Uuid>),
) -> HttpResponse {
    let (state, parameters, organization_id) = data;
    let connection = state.database.get_connection();
    let user = User::new("username", "roles");
    if user.requires_role("Admin").is_err() {
        return application::unauthorized();
    }

    let venue_response = Venue::find(&parameters.id, &*connection);
    match venue_response {
        Ok(venue) => {
            let venue_update_response =
                venue.add_to_organization(&organization_id.into_inner(), &*connection);
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
