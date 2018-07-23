use actix_web::{HttpRequest, Json, Path, Result, State};
use bigneon_db::models::{NewVenue, Venue};
use models::user::User;
use serde_json;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index(state: State<AppState>) -> Result<String> {
    let connection = state.database.get_connection();
    let venue_response = Venue::all(&*connection);
    match venue_response {
        Ok(venues) => Ok(serde_json::to_string(&venues)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> Result<String> {
    let (state, parameters) = data;
    let user = User::new("username", "roles");
    user.requires_role("Guest")?;
    let connection = state.database.get_connection();
    let venue_response = Venue::find(&parameters.id, &*connection);

    match venue_response {
        Ok(venue) => Ok(serde_json::to_string(&venue)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn show_from_organizations(data: (State<AppState>, Json<Uuid>)) -> Result<String> {
    let (state, organization_id) = data;
    let user = User::new("username", "roles");
    user.requires_role("Guest")?;
    let connection = state.database.get_connection();
    let venue_response =
        Venue::find_all_for_organization(&organization_id.into_inner(), &*connection);
    match venue_response {
        Ok(venues) => Ok(serde_json::to_string(&venues)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn create(data: (State<AppState>, Json<NewVenue>)) -> Result<String> {
    let (state, new_venue) = data;
    let connection = state.database.get_connection();
    let venue_response = new_venue.commit(&*connection);

    let user = User::new("username", "roles");
    user.requires_role("Admin")?;
    match venue_response {
        Ok(venue) => Ok(serde_json::to_string(&venue)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn update(data: (State<AppState>, Path<PathParameters>, Json<Venue>)) -> Result<String> {
    let (state, parameters, venue_parameters) = data;
    let connection = state.database.get_connection();
    let user = User::new("username", "roles");
    user.requires_role("Admin")?;

    let updated_venue: Venue = venue_parameters.into_inner();
    let venue_response = Venue::find(&updated_venue.id, &*connection);
    match venue_response {
        Ok(venue) => {
            let venue_update_response = updated_venue.update(&*connection);
            match venue_update_response {
                Ok(updated_val) => Ok(serde_json::to_string(&updated_val)?),
                Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
            }
        }
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn add_to_organization(
    data: (State<AppState>, Path<PathParameters>, Json<Uuid>),
) -> Result<String> {
    let (state, parameters, organization_id) = data;
    let connection = state.database.get_connection();
    let user = User::new("username", "roles");
    user.requires_role("Admin")?;

    let venue_response = Venue::find(&parameters.id, &*connection);
    match venue_response {
        Ok(venue) => {
            let venue_update_response =
                venue.add_to_organization(&organization_id.into_inner(), &*connection);
            match venue_update_response {
                Ok(updated_val) => Ok(serde_json::to_string(&updated_val)?),
                Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
            }
        }
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}
