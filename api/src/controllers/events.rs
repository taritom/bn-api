use actix_web::{http::StatusCode, HttpResponse, Json, Path, Query, State};
use auth::user::User;
use bigneon_db::models::*;
use chrono::prelude::*;
use chrono::Duration;
use db::Connection;
use errors::*;
use models::{PathParameters, RedeemTicketPathParameters, UserDisplayTicketType, WebPayload};
use serde_json::Value;
use serde_with::{self, CommaSeparator};
use server::AppState;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SearchParameters {
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    query: Option<String>,
    region_id: Option<Uuid>,
    #[serde(
        default,
        with = "serde_with::rust::StringWithSeparator::<CommaSeparator>"
    )]
    status: Vec<EventStatus>,
    start_utc: Option<NaiveDateTime>,
    end_utc: Option<NaiveDateTime>,
    page: Option<u32>,
    limit: Option<u32>,
    #[serde(default, deserialize_with = "deserialize_unless_blank")]
    sort: Option<String>,
    dir: Option<SortingDir>,
}

impl From<SearchParameters> for Paging {
    fn from(s: SearchParameters) -> Paging {
        let mut default_tags: HashMap<String, Value> = HashMap::new();
        if let Some(ref i) = s.query {
            default_tags.insert("query".to_owned(), json!(i.clone()));
        }
        if let Some(ref i) = s.region_id {
            default_tags.insert("region_id".to_owned(), json!(i));
        }

        for event_status in s.status.clone().into_iter() {
            default_tags.insert("status".to_owned(), json!(event_status));
        }

        if let Some(ref i) = s.start_utc {
            default_tags.insert("start_utc".to_owned(), json!(i));
        }
        if let Some(ref i) = s.end_utc {
            default_tags.insert("end_utc".to_owned(), json!(i));
        }

        PagingParameters {
            page: s.page,
            limit: s.limit,
            sort: s.sort,
            dir: s.dir,
            tags: default_tags,
        }.into()
    }
}

/**
 * What events does this user have authority to check in
**/
pub fn checkins(
    (conn, query, auth_user): (Connection, Query<SearchParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    auth_user.requires_scope(Scopes::EventScan)?;

    let events = auth_user.user.find_events_with_access_to_scan(conn.get())?;
    let mut payload = Payload::new(events, query.into_inner().into());
    payload.paging.total = payload.data.len() as u64;
    payload.paging.limit = 100;
    Ok(HttpResponse::Ok().json(&payload))
}

pub fn index(
    (connection, query, auth_user): (Connection, Query<SearchParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let query = query.into_inner();

    let user = auth_user.and_then(|auth_user| Some(auth_user.user));
    //TODO remap query to use paging info
    let events = Event::search(
        query.query.clone(),
        query.region_id,
        query.start_utc,
        query.end_utc,
        if query.status.is_empty() {
            None
        } else {
            Some(query.status.clone())
        },
        user,
        connection,
    )?;

    #[derive(Serialize)]
    struct EventVenueEntry {
        id: Uuid,
        name: String,
        organization_id: Uuid,
        venue_id: Option<Uuid>,
        created_at: NaiveDateTime,
        event_start: Option<NaiveDateTime>,
        door_time: Option<NaiveDateTime>,
        status: String,
        publish_date: Option<NaiveDateTime>,
        promo_image_url: Option<String>,
        additional_info: Option<String>,
        top_line_info: Option<String>,
        age_limit: Option<i32>,
        cancelled_at: Option<NaiveDateTime>,
        venue: Option<Venue>,
        min_ticket_price: Option<i64>,
        max_ticket_price: Option<i64>,
    }

    let mut venue_ids: Vec<Uuid> = events
        .iter()
        .filter(|e| e.venue_id.is_some())
        .map(|e| e.venue_id.unwrap())
        .collect();
    venue_ids.sort();
    venue_ids.dedup();

    let venues = Venue::find_by_ids(venue_ids, connection)?;
    let venue_map = venues.into_iter().fold(HashMap::new(), |mut map, v| {
        map.insert(v.id, v.clone());
        map
    });

    let results = events.into_iter().fold(Vec::new(), |mut results, event| {
        results.push(EventVenueEntry {
            venue: event.venue_id.and_then(|v| Some(venue_map[&v].clone())),
            id: event.id,
            name: event.name,
            organization_id: event.organization_id,
            venue_id: event.venue_id,
            created_at: event.created_at,
            event_start: event.event_start,
            door_time: event.door_time,
            status: event.status,
            publish_date: event.publish_date,
            promo_image_url: event.promo_image_url,
            additional_info: event.additional_info,
            top_line_info: event.top_line_info,
            age_limit: event.age_limit,
            cancelled_at: event.cancelled_at,
            min_ticket_price: event.min_ticket_price,
            max_ticket_price: event.max_ticket_price,
        });
        results
    });
    let mut payload = Payload::new(results, query.into());
    payload.paging.total = payload.data.len() as u64;
    payload.paging.limit = 100;
    Ok(HttpResponse::Ok().json(&payload))
}

pub fn show(
    (connection, parameters, user): (Connection, Path<PathParameters>, Option<User>),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    let organization = event.organization(connection)?;
    let fee_schedule = FeeSchedule::find(organization.fee_schedule_id, connection)?;

    let venue = event.venue(connection)?;
    let event_artists = EventArtist::find_all_from_event(event.id, connection)?;
    let total_interest = EventInterest::total_interest(event.id, connection)?;
    let user_interest = match user {
        Some(u) => EventInterest::user_interest(event.id, u.id(), connection)?,
        None => false,
    };

    let ticket_types = TicketType::find_by_event_id(parameters.id, connection)?;
    let mut display_ticket_types = Vec::new();
    for ticket_type in ticket_types {
        let display_ticket_type =
            UserDisplayTicketType::from_ticket_type(&ticket_type, &fee_schedule, connection)?;

        display_ticket_types.push(display_ticket_type);
    }

    //This struct is used to just contain the id and name of the org
    #[derive(Serialize)]
    struct ShortOrganization {
        id: Uuid,
        name: String,
    }

    #[derive(Serialize)]
    struct R {
        id: Uuid,
        name: String,
        organization_id: Uuid,
        venue_id: Option<Uuid>,
        created_at: NaiveDateTime,
        event_start: Option<NaiveDateTime>,
        door_time: Option<NaiveDateTime>,
        fee_in_cents: Option<i64>,
        status: String,
        publish_date: Option<NaiveDateTime>,
        promo_image_url: Option<String>,
        additional_info: Option<String>,
        top_line_info: Option<String>,
        age_limit: Option<i32>,
        video_url: Option<String>,
        organization: ShortOrganization,
        venue: Option<Venue>,
        artists: Vec<DisplayEventArtist>,
        ticket_types: Vec<UserDisplayTicketType>,
        total_interest: u32,
        user_is_interested: bool,
        min_ticket_price: Option<i64>,
        max_ticket_price: Option<i64>,
    }

    Ok(HttpResponse::Ok().json(&R {
        id: event.id,
        name: event.name,
        organization_id: event.organization_id,
        venue_id: event.venue_id,
        created_at: event.created_at,
        event_start: event.event_start,
        door_time: event.door_time,
        fee_in_cents: event.fee_in_cents,
        status: event.status,
        publish_date: event.publish_date,
        promo_image_url: event.promo_image_url,
        additional_info: event.additional_info,
        top_line_info: event.top_line_info,
        age_limit: event.age_limit,
        video_url: event.video_url,
        organization: ShortOrganization {
            id: organization.id,
            name: organization.name,
        },
        venue,
        artists: event_artists,
        ticket_types: display_ticket_types,
        total_interest,
        user_is_interested: user_interest,
        min_ticket_price: event.min_ticket_price,
        max_ticket_price: event.max_ticket_price,
    }))
}

pub fn publish(
    (connection, path, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let conn = connection.get();
    let event = Event::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::EventWrite, &event.organization(conn)?, conn)?;
    event.publish(conn)?;
    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize, Serialize)]
pub struct TicketRedeemRequest {
    pub redeem_key: String,
}

pub fn redeem_ticket(
    (connection, parameters, redeem_parameters, auth_user, state): (
        Connection,
        Path<RedeemTicketPathParameters>,
        Json<TicketRedeemRequest>,
        User,
        State<AppState>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let (event, ticket) = TicketInstance::find_for_processing(
        parameters.ticket_instance_id,
        parameters.id,
        connection,
    )?;
    let db_event = Event::find(event.id, connection)?;
    let organization = db_event.organization(connection)?;
    auth_user.requires_scope_for_organization(Scopes::TicketAdmin, &organization, connection)?;

    let redeemable_ticket =
        match TicketInstance::show_redeemable_ticket(parameters.ticket_instance_id, connection) {
            Ok(redeemable_ticket) => Some(redeemable_ticket),
            Err(_) => None,
        };

    println!("{:?}", redeemable_ticket);
    let result =
        TicketInstance::redeem_ticket(ticket.id, redeem_parameters.redeem_key.clone(), connection);

    match result {
        Ok(r) => {
            match r {
                RedeemResults::TicketRedeemSuccess => {
                    //Redeem ticket on chain
                    let asset = Asset::find(ticket.asset_id, connection)?;
                    match asset.blockchain_asset_id {
                    Some(a) => {
                        let wallet = Wallet::find(ticket.wallet_id, connection)?;
                        state.config.tari_client.modify_asset_redeem_token(&wallet.secret_key, &wallet.public_key,
                                                                           &a,
                                                                           vec![ticket.token_id as u64],
                        )?;

                        Ok(HttpResponse::Ok().json(redeemable_ticket))
                    },
                    None => Ok(HttpResponse::BadRequest().json(json!({ "error": "Could not complete this checkout because the asset has not been assigned on the blockchain.".to_string()}))),
                }
                }
                RedeemResults::TicketAlreadyRedeemed => Ok(HttpResponse::Conflict()
                    .json(json!({"error": "Ticket has already been redeemed.".to_string()}))),
                RedeemResults::TicketInvalid => Ok(HttpResponse::BadRequest()
                    .json(json!({"error": "Ticket is invalid.".to_string()}))),
            }
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(
            json!({ "error": e.cause.unwrap_or("Tickets#Redeem: No cause listed".to_string()),}),
        )),
    }
}

pub fn show_from_organizations(
    (connection, path, paging, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        User,
    ),
) -> Result<WebPayload<EventSummaryResult>, BigNeonError> {
    let conn = connection.get();
    let org = Organization::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::EventWrite, &org, conn)?;

    let events = Event::find_all_events_for_organization(
        path.id,
        paging
            .get_tag("past_or_upcoming")
            .unwrap_or_else(|| "Upcoming".to_string())
            .parse()?,
        paging.page(),
        paging.limit(),
        conn,
    )?;
    Ok(WebPayload::new(StatusCode::OK, events))
}

#[derive(Deserialize)]
pub struct DashboardParameters {
    start_utc: Option<NaiveDate>, // Defaults to 29 days ago if not provided
    end_utc: Option<NaiveDate>,   // Defaults to today if not provided
}

#[derive(Deserialize, Serialize)]
pub struct DashboardResult {
    pub event: EventSummaryResult,
    pub day_stats: Vec<DayStats>,
}

pub fn dashboard(
    (connection, path, query, user): (
        Connection,
        Path<PathParameters>,
        Query<DashboardParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = connection.get();
    let event = Event::find(path.id, conn)?;
    user.requires_scope_for_organization(Scopes::EventWrite, &event.organization(conn)?, conn)?;
    let summary = event.summary(conn)?;
    let start_utc = query
        .start_utc
        .unwrap_or(Utc::now().naive_utc().date() - Duration::days(29));
    let end_utc = query.end_utc.unwrap_or(Utc::now().naive_utc().date());

    let day_stats = event.get_sales_by_date_range(start_utc, end_utc, conn)?;

    Ok(HttpResponse::Ok().json(DashboardResult {
        event: summary,
        day_stats,
    }))
}

pub fn show_from_venues(
    (connection, venue_id, query_parameters): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let events = Event::find_all_events_for_venue(&venue_id.id, connection.get())?;
    let payload = Payload::from_data(events, query_parameters.page(), query_parameters.limit());
    Ok(HttpResponse::Ok().json(&payload))
}

#[derive(Deserialize, Debug)]
pub struct AddArtistRequest {
    pub artist_id: Uuid,
    pub rank: i32,
    pub set_time: Option<NaiveDateTime>,
}

pub fn create(
    (connection, new_event, user): (Connection, Json<NewEvent>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let organization = Organization::find(new_event.organization_id, connection)?;
    user.requires_scope_for_organization(Scopes::EventWrite, &organization, connection)?;

    let event = new_event.commit(connection)?;
    Ok(HttpResponse::Created().json(&event))
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateArtistsRequest {
    pub artist_id: Uuid,
    pub set_time: Option<NaiveDateTime>,
}

#[derive(Deserialize, Debug, Default)]
pub struct UpdateArtistsRequestList {
    pub artists: Vec<UpdateArtistsRequest>,
}

pub fn update(
    (connection, parameters, event_parameters, user): (
        Connection,
        Path<PathParameters>,
        Json<EventEditableAttributes>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    let organization = event.organization(connection)?;
    user.requires_scope_for_organization(Scopes::EventWrite, &organization, connection)?;

    let updated_event = event.update(event_parameters.into_inner(), connection)?;
    Ok(HttpResponse::Ok().json(&updated_event))
}

pub fn cancel(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    let organization = event.organization(connection)?;
    user.requires_scope_for_organization(Scopes::EventWrite, &organization, connection)?;

    //Doing this in the DB layer so it can use the DB time as now.
    let updated_event = event.cancel(connection)?;

    Ok(HttpResponse::Ok().json(&updated_event))
}

pub fn list_interested_users(
    (connection, path_parameters, query, user): (
        Connection,
        Path<PathParameters>,
        Query<PagingParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::EventInterest)?;

    let connection = connection.get();
    let paging: Paging = query.clone().into();

    let payload = EventInterest::list_interested_users(
        path_parameters.id,
        user.id(),
        paging.page * paging.limit,
        (paging.page * paging.limit) + paging.limit,
        connection,
    )?;
    Ok(HttpResponse::Ok().json(&payload))
}

pub fn add_interest(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::EventInterest)?;

    let connection = connection.get();
    let event_interest = EventInterest::create(parameters.id, user.id()).commit(connection)?;
    Ok(HttpResponse::Created().json(&event_interest))
}

pub fn remove_interest(
    (connection, parameters, user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    user.requires_scope(Scopes::EventInterest)?;

    let connection = connection.get();
    let event_interest = EventInterest::remove(parameters.id, user.id(), connection)?;
    Ok(HttpResponse::Ok().json(&event_interest))
}

pub fn add_artist(
    (connection, parameters, event_artist, user): (
        Connection,
        Path<PathParameters>,
        Json<AddArtistRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    let organization = event.organization(connection)?;
    user.requires_scope_for_organization(Scopes::EventWrite, &organization, connection)?;

    let event_artist = EventArtist::create(
        parameters.id,
        event_artist.artist_id,
        event_artist.rank,
        event_artist.set_time,
    ).commit(connection)?;
    Ok(HttpResponse::Created().json(&event_artist))
}

pub fn update_artists(
    (connection, parameters, artists, user): (
        Connection,
        Path<PathParameters>,
        Json<UpdateArtistsRequestList>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let event = Event::find(parameters.id, connection)?;
    let organization = event.organization(connection)?;
    user.requires_scope_for_organization(Scopes::EventWrite, &organization, connection)?;

    EventArtist::clear_all_from_event(parameters.id, connection)?;

    let mut rank = 0;
    let mut added_artists: Vec<EventArtist> = Vec::new();

    for a in &artists.into_inner().artists {
        added_artists.push(
            EventArtist::create(parameters.id, a.artist_id, rank, a.set_time).commit(connection)?,
        );
        rank += 1;
    }

    Ok(HttpResponse::Ok().json(&added_artists))
}

#[derive(Deserialize)]
pub struct GuestListQueryParameters {
    pub query: String,
}

impl From<GuestListQueryParameters> for Paging {
    fn from(s: GuestListQueryParameters) -> Paging {
        let mut default_tags: HashMap<String, Value> = HashMap::new();
        default_tags.insert("query".to_owned(), json!(s.query.clone()));

        Paging {
            page: 0,
            limit: 100,
            sort: "".to_owned(),
            dir: SortingDir::Asc,
            total: 0,
            tags: default_tags,
        }
    }
}

pub fn guest_list(
    (connection, query, path, user): (
        Connection,
        Query<GuestListQueryParameters>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    //TODO refactor GuestListQueryParameters to PagingParameters
    let conn = connection.get();
    let event = Event::find(path.id, conn)?;
    user.requires_scope_for_organization(
        Scopes::EventViewGuests,
        &event.organization(conn)?,
        conn,
    )?;
    let tickets = event.guest_list(&query.query, conn)?;
    let payload = Payload::new(tickets, query.into_inner().into());
    Ok(HttpResponse::Ok().json(payload))
}

pub fn codes(
    (conn, query, path, user): (
        Connection,
        Query<PagingParameters>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    user.requires_scope_for_organization(
        Scopes::CodeRead,
        &Organization::find_for_event(path.id, conn)?,
        conn,
    )?;

    let mut code_type: Option<CodeTypes> = None;
    if let Some(value) = query.tags.get("type") {
        code_type = serde_json::from_value(value.clone())?;
    }

    //TODO: remap query to use paging info
    let codes = Code::find_for_event(path.id, code_type, conn)?;
    let mut payload = Payload::from_data(codes, query.page(), query.limit());
    payload.paging.tags = query.tags.clone();

    Ok(HttpResponse::Ok().json(payload))
}

pub fn holds(
    (conn, query, path, user): (
        Connection,
        Query<PagingParameters>,
        Path<PathParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let conn = conn.get();
    user.requires_scope_for_organization(
        Scopes::HoldRead,
        &Organization::find_for_event(path.id, conn)?,
        conn,
    )?;
    let holds = Hold::find_for_event(path.id, conn)?;

    #[derive(Serialize)]
    struct R {
        pub id: Uuid,
        pub name: String,
        pub event_id: Uuid,
        pub redemption_code: String,
        pub discount_in_cents: Option<i64>,
        pub end_at: Option<NaiveDateTime>,
        pub max_per_order: Option<i64>,
        pub hold_type: String,
        pub ticket_type_id: Uuid,
        pub available: u32,
        pub quantity: u32,
    }

    let mut list = Vec::<R>::new();
    for hold in holds {
        let (quantity, available) = hold.quantity(conn)?;

        let r = R {
            id: hold.id,
            name: hold.name,
            event_id: hold.event_id,
            redemption_code: hold.redemption_code,
            discount_in_cents: hold.discount_in_cents,
            end_at: hold.end_at,
            max_per_order: hold.max_per_order,
            hold_type: hold.hold_type,
            ticket_type_id: hold.ticket_type_id,
            available,
            quantity,
        };

        list.push(r);
    }

    Ok(HttpResponse::Ok().json(Payload::from_data(list, query.page(), query.limit())))
}
