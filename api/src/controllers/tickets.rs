use actix_web::{HttpResponse, Path};
use auth::user::User;
use bigneon_db::models::*;
use db::Connection;
use errors::*;
use helpers::application;
use models::{OptionalPathParameters, PathParameters};

pub fn index(
    (connection, parameters, auth_user): (Connection, Path<OptionalPathParameters>, User),
) -> Result<HttpResponse, BigNeonError> {
    let connection = connection.get();
    let tickets = TicketInstance::find_for_user(auth_user.id(), parameters.id, connection)?;

    // If specifying event drill into tuple vector to return tickets alone
    if parameters.id.is_some() && !tickets.is_empty() {
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
    let (event, user, ticket) = TicketInstance::find(parameters.id, connection)?;
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
