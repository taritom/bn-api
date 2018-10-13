use actix_web::{http::StatusCode, HttpResponse, Json, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use chrono::prelude::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::{OptionalPathParameters, Paging, PathParameters, Payload, SearchParam, SortingDir};

#[derive(Deserialize)]
pub struct SearchParameters {
    pub start_utc: Option<NaiveDateTime>,
    pub end_utc: Option<NaiveDateTime>,
}
impl SearchParameters {
    pub fn create_paging_struct(&self) -> Paging {
        let mut default_tags = Vec::new();
        if let Some(ref i) = self.start_utc {
            let new_value = SearchParam {
                name: "start_utc".to_owned(),
                values: vec![i.to_string()],
            };
            default_tags.push(new_value);
        }
        if let Some(ref i) = self.end_utc {
            let new_value = SearchParam {
                name: "end_utc".to_owned(),
                values: vec![i.to_string()],
            };
            default_tags.push(new_value);
        }
        Paging {
            page: 0,
            limit: 100,
            sort: "".to_owned(),
            dir: SortingDir::None,
            total: 0,
            tags: default_tags,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TicketRedeemRequest {
    pub redeem_key: String,
}

pub fn index(
    (connection, path, parameters, auth_user): (
        Connection,
        Path<OptionalPathParameters>,
        Query<SearchParameters>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    //todo convert to use pagingparams

    let connection = connection.get();
    let queryparms = parameters.create_paging_struct();
    let tickets = TicketInstance::find_for_user(
        auth_user.id(),
        path.id,
        parameters.start_utc,
        parameters.end_utc,
        connection,
    )?;
    let ticket_count = tickets.len();
    let mut payload = Payload {
        data: tickets,
        paging: Paging::clone_with_new_total(&queryparms, ticket_count as u64),
    };
    payload.paging.limit = ticket_count as u64;
    // If specifying event drill into tuple vector to return tickets alone
    if path.id.is_some() && !payload.data.is_empty() {
        let mut payload2 = Payload {
            data: (payload.data[0].1).clone(),
            paging: Paging::clone_with_new_total(&queryparms, ticket_count as u64),
        };
        payload2.paging.limit = payload2.data.len() as u64;
        payload2.paging.total = payload2.data.len() as u64;
        return Ok(HttpResponse::Ok().json(&payload2));
    }

    Ok(HttpResponse::Ok().json(&payload))
}

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct ShowTicketResponse {
    pub event: DisplayEvent,
    pub user: DisplayUser,
    pub ticket: DisplayTicket,
}

pub fn show(
    (connection, parameters, auth_user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let (event, user, ticket) = TicketInstance::find_for_display(parameters.id, connection)?;
    let db_event = Event::find(event.id, connection)?;
    let organization = db_event.organization(connection)?;

    if !auth_user.has_scope(Scopes::TicketAdmin, Some(&organization), connection)?
        && user.id != auth_user.id()
    {
        return application::unauthorized();
    }

    let ticket_response = ShowTicketResponse {
        event,
        user,
        ticket,
    };

    Ok(HttpResponse::Ok().json(&ticket_response))
}

pub fn redeem(
    (connection, parameters, redeem_parameters, auth_user): (
        Connection,
        Path<PathParameters>,
        Json<TicketRedeemRequest>,
        User,
    ),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let (event, _user, ticket) = TicketInstance::find_for_display(parameters.id, connection)?;
    let db_event = Event::find(event.id, connection)?;
    let organization = db_event.organization(connection)?;

    if !auth_user.has_scope(Scopes::TicketAdmin, Some(&organization), connection)? {
        return application::unauthorized();
    }

    let result =
        TicketInstance::redeem_ticket(ticket.id, redeem_parameters.redeem_key.clone(), connection);

    match result {
        Ok(r) => match r {
                RedeemResults::TicketRedeemSuccess => Ok(HttpResponse::Ok().json(json!({"success": true,}))),
                RedeemResults::TicketAlreadyRedeemed => Ok(HttpResponse::Ok().json(json!({"success": false, "message": "Ticket has already been redeemed.".to_string()}))),
                RedeemResults::TicketInvalid => Ok(HttpResponse::Ok().json(json!({"success": false, "message": "Ticket is invalid.".to_string()}))),
            },
        Err(e) => Ok(HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            .into_builder()
            .json(json!({"error": e.cause.unwrap().to_string(),}))),
    }
}

pub fn show_redeemable_ticket(
    (connection, parameters, auth_user): (Connection, Path<PathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let (event, user, _ticket) = TicketInstance::find_for_display(parameters.id, connection)?;
    let db_event = Event::find(event.id, connection)?;
    let organization = db_event.organization(connection)?;

    if !auth_user.has_scope(Scopes::TicketAdmin, Some(&organization), connection)?
        && user.id != auth_user.id()
    {
        return application::unauthorized();
    }

    let redeemable_ticket = TicketInstance::show_redeemable_ticket(parameters.id, connection)?;

    Ok(HttpResponse::Ok().json(&redeemable_ticket))
}
