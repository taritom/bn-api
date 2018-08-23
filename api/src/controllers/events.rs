use actix_web::Query;
use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::*;
use chrono::NaiveDateTime;
use errors::*;
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

pub fn index(
    (state, parameters): (State<AppState>, Query<SearchParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let parameters = parameters.into_inner();
    let events = Event::search(
        parameters.query,
        parameters.start_utc,
        parameters.end_utc,
        &*connection,
    )?;
    Ok(HttpResponse::Ok().json(&events))
}

pub fn show(
    (state, parameters): (State<AppState>, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let event = Event::find(parameters.id, &*connection)?;
    Ok(HttpResponse::Ok().json(&event))
}

pub fn show_from_organizations(
    (state, organization_id): (State<AppState>, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();

    let events = Event::find_all_events_from_organization(&organization_id.id, &*connection)?;
    Ok(HttpResponse::Ok().json(&events))
}

pub fn show_from_venues(
    (state, venue_id): (State<AppState>, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();

    let events = Event::find_all_events_from_venue(&venue_id.id, &*connection)?;
    Ok(HttpResponse::Ok().json(&events))
}

pub fn create(
    (state, new_event, user): (State<AppState>, Json<NewEvent>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    if !user.has_scope(Scopes::EventWrite) {
        return application::unauthorized();
    }
    let event = new_event.commit(&*connection)?;
    Ok(HttpResponse::Created().json(&event))
}

pub fn update(
    (state, parameters, event_parameters, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<EventEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::EventWrite) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    let event = Event::find(parameters.id, &*connection)?;
    let updated_event = event.update(event_parameters.into_inner(), &*connection)?;
    Ok(HttpResponse::Ok().json(&updated_event))
}

pub fn add_interest(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::EventInterest) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    let event_interest = EventInterest::create(parameters.id, user.id()).commit(&*connection)?;
    Ok(HttpResponse::Created().json(&event_interest))
}

pub fn remove_interest(
    (state, parameters, user): (State<AppState>, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::EventInterest) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    let event_interest = EventInterest::remove(parameters.id, user.id(), &*connection)?;
    Ok(HttpResponse::Ok().json(&event_interest))
}

pub fn create_tickets(
    (state, path, data, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<CreateTicketAllocationRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::TicketAdmin) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();
    let event = Event::find(path.id, &*connection)?;
    let organization = event.organization(&*connection)?;
    if !organization.is_member(&user.user, &*connection)? {
        return application::forbidden("User does not belong to this organization");
    }

    let mut allocation =
        TicketAllocation::create(path.id, data.tickets_delta).commit(&*connection)?;

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

    let updated_allocation = allocation.update(&*connection)?;
    Ok(HttpResponse::Ok().json(json!({"ticket_allocation_id": updated_allocation.id})))
}
