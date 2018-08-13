use actix_web::Query;
use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::{Event, EventEditableAttributes, NewEvent, Roles};
use chrono::NaiveDateTime;
use errors::database_error::ConvertToWebError;
use helpers::application;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct SearchParameters {
    name: Option<String>,
    venue: Option<String>,
    artist: Option<String>,
    start_utc: Option<NaiveDateTime>,
    end_utc: Option<NaiveDateTime>,
}

pub fn index(
    (state, query, user): (State<AppState>, Query<SearchParameters>, User),
) -> HttpResponse {
    let connection = state.database.get_connection();
    if !user.has_scope(Scopes::EventRead) {
        return application::unauthorized();
    }
    let query = query.into_inner();
    let event_response = Event::search(
        query.name,
        query.venue,
        query.artist,
        query.start_utc,
        query.end_utc,
        &*connection,
    );
    match event_response {
        Ok(events) => HttpResponse::Ok().json(&events),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>, User)) -> HttpResponse {
    let (state, parameters, user) = data;

    let connection = state.database.get_connection();
    let event_response = Event::find(&parameters.id, &*connection);
    if !user.has_scope(Scopes::EventRead) {
        return application::unauthorized();
    }
    match event_response {
        Ok(event) => HttpResponse::Ok().json(&event),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show_from_organizations(
    data: (State<AppState>, Path<PathParameters>, User),
) -> HttpResponse {
    let (state, organization_id, user) = data;

    let connection = state.database.get_connection();
    if !user.has_scope(Scopes::EventRead) {
        return application::unauthorized();
    }
    let event_response =
        Event::find_all_events_from_organization(&organization_id.id, &*connection);
    match event_response {
        Ok(events) => HttpResponse::Ok().json(&events),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show_from_venues(data: (State<AppState>, Path<PathParameters>, User)) -> HttpResponse {
    let (state, venue_id, user) = data;

    let connection = state.database.get_connection();
    if !user.has_scope(Scopes::EventRead) {
        return application::unauthorized();
    }
    let event_response = Event::find_all_events_from_venue(&venue_id.id, &*connection);
    match event_response {
        Ok(events) => HttpResponse::Ok().json(&events),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn create((state, new_event, user): (State<AppState>, Json<NewEvent>, User)) -> HttpResponse {
    let connection = state.database.get_connection();
    if !user.has_scope(Scopes::EventWrite) {
        return application::unauthorized();
    }
    let event_response = new_event.commit(&*connection);
    match event_response {
        Ok(event) => HttpResponse::Created().json(&event),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn update(
    (state, parameters, event_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<EventEditableAttributes>,
        User,
    ),
) -> HttpResponse {
    if !user.has_scope(Scopes::EventWrite) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    match Event::find(&parameters.id, &*connection) {
        Ok(event) => match event.update(event_parameters.into_inner(), &*connection) {
            Ok(updated_event) => HttpResponse::Ok().json(&updated_event),
            Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
        },
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}
