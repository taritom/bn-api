use actix_web::{http::StatusCode, HttpResponse, Json, Path, Query};
use auth::user::User;
use bigneon_db::models::*;
use chrono::prelude::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::{OptionalPathParameters, PathParameters};

#[derive(Deserialize)]
pub struct SearchParameters {
    pub start_utc: Option<NaiveDateTime>,
    pub end_utc: Option<NaiveDateTime>,
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
    let connection = connection.get();
    let tickets = TicketInstance::find_for_user(
        auth_user.id(),
        path.id,
        parameters.start_utc,
        parameters.end_utc,
        connection,
    )?;

    // If specifying event drill into tuple vector to return tickets alone
    if path.id.is_some() && !tickets.is_empty() {
        return Ok(HttpResponse::Ok().json(&tickets[0].1));
    }

    Ok(HttpResponse::Ok().json(&tickets))
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

pub fn show_redeem_key(
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

    let redeem_key = TicketInstance::show_redeem_key(parameters.id, connection)?;

    if redeem_key.is_some() {
        Ok(HttpResponse::Ok().json(json!({"success": true,"redeem_key": redeem_key.unwrap(),})))
    } else {
        Ok(HttpResponse::Ok()
            .json(json!({"success": false, "message": "Redeem key is not available".to_string()})))
    }
}
