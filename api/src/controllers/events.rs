use actix_web::Query;
use actix_web::{HttpResponse, Json, Path, State};
use auth::user::Scopes;
use auth::user::User;
use bigneon_db::models::*;
use chrono::NaiveDateTime;
use errors::*;
use helpers::application;
use models::{CreateTicketAllocationRequest, DisplayTicketType};
use server::AppState;
use tari::tariclient::*;
use uuid::Uuid;
use validator::Validate;

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

#[derive(Deserialize, Debug)]
pub struct AddArtistRequest {
    pub artist_id: Uuid,
    pub rank: i32,
    pub set_time: Option<NaiveDateTime>,
}

#[derive(Deserialize, Validate)]
pub struct CreateEventRequest {
    pub name: String,
    pub organization_id: Uuid,
    pub venue_id: Option<Uuid>,
    pub event_start: Option<NaiveDateTime>,
    pub door_time: Option<NaiveDateTime>,
    pub publish_date: Option<NaiveDateTime>,
    #[validate(url)]
    pub promo_image_url: Option<String>,
    pub additional_info: Option<String>,
    pub age_limit: Option<i32>,
}

#[derive(Deserialize, Debug)]
pub struct UpdateArtistsRequest {
    pub artist_id: Uuid,
    pub set_time: Option<NaiveDateTime>,
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
    (state, parameters, user): (State<AppState>, Path<PathParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    let event = Event::find(parameters.id, &*connection)?;
    let organization = event.organization(&*connection)?;
    let venue = event.venue(&*connection)?;
    let total_interest = EventInterest::total_interest(event.id, &*connection)?;
    let event_artists = EventArtist::find_all_from_event(event.id, &*connection)?;

    let user_interest = match user {
        Some(u) => EventInterest::user_interest(event.id, u.id(), &*connection)?,
        None => false,
    };

    //This struct is used to just contain the id and name of the org
    #[derive(Serialize)]
    struct ShortOrganization {
        id: Uuid,
        name: String,
    }

    #[derive(Serialize)]
    struct R {
        event: Event,
        organization: ShortOrganization,
        venue: Option<Venue>,
        artists: Vec<EventArtist>,
        total_interest: u32,
        user_is_interested: bool,
    }

    Ok(HttpResponse::Ok().json(&R {
        event: event,
        organization: ShortOrganization {
            id: organization.id,
            name: organization.name,
        },
        venue: venue,
        artists: event_artists,
        total_interest: total_interest,
        user_is_interested: user_interest,
    }))
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
    (state, new_event, user): (State<AppState>, Json<CreateEventRequest>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = state.database.get_connection();
    if !user.has_scope(Scopes::EventWrite) {
        return application::unauthorized();
    }

    match new_event.validate() {
        Ok(_) => {
            let event_response = Event::create(
                new_event.name.as_str(),
                new_event.organization_id,
                new_event.venue_id,
                new_event.event_start,
                new_event.door_time,
                new_event.publish_date,
            ).commit(&*connection)?;
            Ok(HttpResponse::Created().json(&event_response))
        }
        Err(e) => application::validation_error_response(e),
    }
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

    match event_parameters.validate() {
        Ok(_) => {
            let updated_event = event.update(event_parameters.into_inner(), &*connection)?;
            Ok(HttpResponse::Ok().json(&updated_event))
        }
        Err(e) => application::validation_error_response(e),
    }
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

pub fn add_artist(
    (state, parameters, event_artist, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<AddArtistRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::EventWrite) {
        return application::unauthorized();
    }

    let connection = state.database.get_connection();
    let event_artist = EventArtist::create(
        parameters.id,
        event_artist.artist_id,
        event_artist.rank,
        event_artist.set_time,
    ).commit(&*connection)?;
    Ok(HttpResponse::Created().json(&event_artist))
}

pub fn update_artists(
    (state, parameters, artists, user): (
        State<AppState>,
        Path<PathParameters>,
        Json<Vec<UpdateArtistsRequest>>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    if !user.has_scope(Scopes::EventWrite) {
        return application::unauthorized();
    }
    let connection = state.database.get_connection();

    EventArtist::clear_all_from_event(parameters.id, &*connection)?;

    let mut rank = 0;
    let mut added_artists: Vec<EventArtist> = Vec::new();

    for a in &artists.into_inner() {
        added_artists.push(
            EventArtist::create(parameters.id, a.artist_id, rank, a.set_time).commit(&*connection)?,
        );
        rank += 1;
    }

    Ok(HttpResponse::Ok().json(&added_artists))
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

    let asset_id = match tari_client.create_asset(Asset {
        id: data.name.clone(),
        name: data.name.clone(),
        symbol: "sym".into(), //TODO remove symbol from asset spec
        decimals: 0,
        total_supply: data.tickets_delta,
        authorised_signers: vec!["896asudh9872ty4".into()], //TODO add bn-api pub key here
        issuer: "BigNeonAddress".into(),
        valid: true,
        rule_flags: 0,
        rule_metadata: "".into(),
        expire_date: 10,
    }) {
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

pub fn list_ticket_types(
    (state, path): (State<AppState>, Path<PathParameters>),
) -> Result<HttpResponse, BigNeonError> {
    let conn = state.database.get_connection();
    let ticket_types = TicketType::find_by_event_id(path.id, &*conn)?;
    let mut encoded_ticket_types = Vec::<DisplayTicketType>::new();
    for t in ticket_types {
        encoded_ticket_types.push(DisplayTicketType::from_ticket_type(&t, &*conn)?);
    }

    #[derive(Serialize)]
    struct R {
        ticket_types: Vec<DisplayTicketType>,
    };

    Ok(HttpResponse::Ok().json(R {
        ticket_types: encoded_ticket_types,
    }))
}
