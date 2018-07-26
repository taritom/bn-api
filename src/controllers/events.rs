use actix_web::{HttpResponse, Json, Path, State};
use auth::user::User;
use bigneon_db::models::{Event, NewEvent, Roles};
use errors::database_error::ConvertToWebError;
use helpers::application;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

pub fn index((state, user): (State<AppState>, User)) -> HttpResponse {
    let connection = state.database.get_connection();
    let event_response = Event::all(&*connection);
    if !user.is_in_role(Roles::Guest) {
        return application::unauthorized();
    }
    match event_response {
        Ok(events) => HttpResponse::Ok().json(&events),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>, User)) -> HttpResponse {
    let (state, parameters, user) = data;

    let connection = state.database.get_connection();
    let event_response = Event::find(&parameters.id, &*connection);
    if !user.is_in_role(Roles::Guest) {
        return application::unauthorized();
    }
    match event_response {
        Ok(event) => HttpResponse::Ok().json(&event),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show_from_organizations(data: (State<AppState>, Json<Uuid>, User)) -> HttpResponse {
    let (state, organization_id, user) = data;

    let connection = state.database.get_connection();
    if !user.is_in_role(Roles::Guest) {
        return application::unauthorized();
    }
    let event_response =
        Event::find_all_events_from_organization(&organization_id.into_inner(), &*connection);
    match event_response {
        Ok(events) => HttpResponse::Ok().json(&events),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show_from_venues(data: (State<AppState>, Json<Uuid>, User)) -> HttpResponse {
    let (state, venue_id, user) = data;

    let connection = state.database.get_connection();
    if !user.is_in_role(Roles::Guest) {
        return application::unauthorized();
    }
    let event_response = Event::find_all_events_from_venue(&venue_id.into_inner(), &*connection);
    match event_response {
        Ok(events) => HttpResponse::Ok().json(&events),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn create((state, new_event, user): (State<AppState>, Json<NewEvent>, User)) -> HttpResponse {
    let connection = state.database.get_connection();
    let event_response = new_event.commit(&*connection);
    if !user.is_in_role(Roles::OrgOwner) {
        return application::unauthorized();
    }
    match event_response {
        Ok(event) => HttpResponse::Created().json(&event),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn update(
    (state, parameters, event_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<Event>,
        User,
    ),
) -> HttpResponse {
    let connection = state.database.get_connection();
    if !user.is_in_role(Roles::OrgOwner) {
        return application::unauthorized();
    }

    let updated_event: Event = event_parameters.into_inner();
    let event_response = Event::find(&parameters.id, &*connection);
    match event_response {
        Ok(_event) => {
            let event_update_response = updated_event.update(&*connection);
            match event_update_response {
                Ok(updated_event) => HttpResponse::Ok().json(&updated_event),
                Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
            }
        }
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}
