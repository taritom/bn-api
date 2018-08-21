use actix_web::Query;
use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::*;
use chrono::NaiveDateTime;
use errors::database_error::ConvertToWebError;
use helpers::application;
use models::CreateTicketAllocationRequest;
use server::AppState;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PathParameters {
    pub id: Uuid,
}

#[derive(Deserialize)]
pub struct SearchParameters {
    query: Option<String>,
    start_utc: Option<NaiveDateTime>,
    end_utc: Option<NaiveDateTime>,
}

pub fn index((state, parameters): (State<AppState>, Query<SearchParameters>)) -> HttpResponse {
    let connection = state.database.get_connection();
    let parameters = parameters.into_inner();
    let event_response = Event::search(
        parameters.query,
        parameters.start_utc,
        parameters.end_utc,
        &*connection,
    );
    match event_response {
        Ok(events) => HttpResponse::Ok().json(&events),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show(data: (State<AppState>, Path<PathParameters>)) -> HttpResponse {
    let (state, parameters) = data;

    let connection = state.database.get_connection();
    let event_response = Event::find(parameters.id, &*connection);
    match event_response {
        Ok(event) => HttpResponse::Ok().json(&event),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show_from_organizations(
    (state, organization_id): (State<AppState>, Path<PathParameters>),
) -> HttpResponse {
    let connection = state.database.get_connection();

    let event_response =
        Event::find_all_events_from_organization(&organization_id.id, &*connection);
    match event_response {
        Ok(events) => HttpResponse::Ok().json(&events),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn show_from_venues(
    (state, venue_id): (State<AppState>, Path<PathParameters>),
) -> HttpResponse {
    let connection = state.database.get_connection();

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
    match Event::find(parameters.id, &*connection) {
        Ok(event) => match event.update(event_parameters.into_inner(), &*connection) {
            Ok(updated_event) => HttpResponse::Ok().json(&updated_event),
            Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
        },
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn add_interest(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> HttpResponse {
    if !user.has_scope(Scopes::EventInterest) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    let event_interest_response =
        EventInterest::create(parameters.id, user.id()).commit(&*connection);
    match event_interest_response {
        Ok(event_interest) => HttpResponse::Created().json(&event_interest),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn remove_interest(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> HttpResponse {
    if !user.has_scope(Scopes::EventInterest) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    let event_interest_response = EventInterest::remove(parameters.id, user.id(), &*connection);
    match event_interest_response {
        Ok(event_interest) => HttpResponse::Ok().json(&event_interest),
        Err(e) => HttpResponse::from_error(ConvertToWebError::create_http_error(&e)),
    }
}

pub fn create_tickets(
    (state, path, data, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<CreateTicketAllocationRequest>,
        User,
    ),
) -> HttpResponse {
    if !user.has_scope(Scopes::TicketAdmin) {
        return application::unauthorized();
    }
    let conn = state.database.get_connection();
    let event = Event::find(path.id, &*conn);

    let event = match event {
        Ok(e) => e,
        Err(e) => return e.to_response(),
    };

    let org = match event.organization(&*conn) {
        Ok(o) => o,
        Err(e) => return e.to_response(),
    };

    match org.is_member(&user.user, &*conn) {
        Ok(b) => if !b {
            return application::forbidden("User does not belong to this organization");
        },
        Err(e) => return e.to_response(),
    };

    let mut allocation = match TicketAllocation::create(path.id, data.tickets_delta).commit(&*conn)
    {
        Ok(a) => a,
        Err(e) => return e.to_response(),
    };

    // TODO: move this to an async processor...
    let tari_client = state.get_tari_client();

    let asset_id = match tari_client.create_asset(
        &data.name,
        &"TIX",
        0,
        data.tickets_delta,
        &"BigNeonAddress",
    ) {
        Ok(a) => a,
        Err(e) => {
            return application::internal_server_error(&format!(
                "Could not create tari asset:{}",
                e.to_string()
            ))
        }
    };

    allocation.set_asset_id(asset_id);

    match allocation.update(&*conn) {
        Ok(a) => return HttpResponse::Ok().json(json!({"ticket_allocation_id": a.id})),
        Err(e) => return e.to_response(),
    }
}
