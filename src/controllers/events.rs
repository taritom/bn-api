use actix_web::{HttpRequest, Json, Path, Result, State};
use bigneon_db::models::{Event, NewEvent};
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
    let event_response = Event::all(&*connection);
    match event_response {
        Ok(events) => Ok(serde_json::to_string(&events)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> Result<String> {
    let (state, parameters) = data;
    let user = User::new("username", "roles");
    user.requires_role("Guest")?;
    let connection = state.database.get_connection();
    let event_response = Event::find(&parameters.id, &*connection);

    match event_response {
        Ok(event) => Ok(serde_json::to_string(&event)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn show_from_organizations(data: (State<AppState>, Json<Uuid>)) -> Result<String> {
    let (state, organization_id) = data;
    let user = User::new("username", "roles");
    user.requires_role("Guest")?;
    let connection = state.database.get_connection();
    let event_response =
        Event::find_all_events_from_organization(&organization_id.into_inner(), &*connection);
    match event_response {
        Ok(events) => Ok(serde_json::to_string(&events)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn show_from_venues(data: (State<AppState>, Json<Uuid>)) -> Result<String> {
    let (state, venue_id) = data;
    let user = User::new("username", "roles");
    user.requires_role("Guest")?;
    let connection = state.database.get_connection();
    let event_response = Event::find_all_events_from_venue(&venue_id.into_inner(), &*connection);
    match event_response {
        Ok(events) => Ok(serde_json::to_string(&events)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn create(data: (State<AppState>, Json<NewEvent>)) -> Result<String> {
    let (state, new_event) = data;
    let connection = state.database.get_connection();
    let event_response = new_event.commit(&*connection);

    let user = User::new("username", "roles");
    user.requires_role("Admin")?;
    match event_response {
        Ok(event) => Ok(serde_json::to_string(&event)?),
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}

pub fn update(data: (State<AppState>, Path<PathParameters>, Json<Event>)) -> Result<String> {
    let (state, parameters, event_parameters) = data;
    let connection = state.database.get_connection();
    let user = User::new("username", "roles");
    user.requires_role("Admin")?;

    let updated_event: Event = event_parameters.into_inner();
    let event_response = Event::find(&updated_event.id, &*connection);
    match event_response {
        Ok(event) => {
            let event_update_response = updated_event.update(&*connection);
            match event_update_response {
                Ok(updated_val) => Ok(serde_json::to_string(&updated_val)?),
                Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
            }
        }
        Err(_e) => Ok("{\"error\": \"An error has occurred\"}".to_string()),
    }
}
