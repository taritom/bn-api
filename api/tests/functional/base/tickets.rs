use actix_web::{http::StatusCode, FromRequest, Json, Path};
use bigneon_api::controllers::tickets::{self, ShowTicketResponse, TicketRedeemRequest};
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn show_other_user_ticket(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let request = TestRequest::create();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let event = database
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let user2 = database.create_user().finish();
    let mut cart = Order::create(user2.id, OrderTypes::Cart)
        .commit(&database.connection)
        .unwrap();
    let ticket_type = &event.ticket_types(&database.connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 1, &database.connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(&database.connection).unwrap();
    cart.add_external_payment("test".to_string(), user.id, total, &database.connection)
        .unwrap();

    let mut path = Path::<PathParameters>::extract(&request.request).unwrap();
    path.id = ticket.id;

    let response = tickets::show((database.connection.clone().into(), path, auth_user)).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let body = support::unwrap_body_to_string(&response).unwrap();
        let ticket_response: ShowTicketResponse = serde_json::from_str(&body).unwrap();
        let expected_ticket = DisplayTicket {
            id: ticket.id,
            ticket_type_name: ticket_type.name.clone(),
        };

        let expected_result = ShowTicketResponse {
            ticket: expected_ticket,
            user: user2.into(),
            event: event.for_display(&database.connection).unwrap(),
        };
        assert_eq!(expected_result, ticket_response);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn redeem_ticket(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let request = TestRequest::create();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let event = database
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let user2 = database.create_user().finish();
    let mut cart = Order::create(user2.id, OrderTypes::Cart)
        .commit(&database.connection)
        .unwrap();
    let ticket_type = &event.ticket_types(&database.connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 1, &database.connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(&database.connection).unwrap();
    cart.add_external_payment("test".to_string(), user.id, total, &database.connection)
        .unwrap();

    let ticket = TicketInstance::find(ticket.id, &database.connection).unwrap();

    let mut path = Path::<PathParameters>::extract(&request.request).unwrap();
    path.id = ticket.id;
    let mut path2 = Path::<PathParameters>::extract(&request.request).unwrap();
    path2.id = ticket.id;

    //First try when Redeem code is wrong
    let request_data = TicketRedeemRequest {
        redeem_key: "WrongKey".to_string(),
    };

    let response = tickets::redeem((
        database.connection.clone().into(),
        path,
        Json(request_data),
        auth_user.clone(),
        request.extract_state(),
    )).unwrap();

    #[derive(Deserialize)]

    struct R {
        success: bool,
        //message: Option<String>,
    }

    if should_test_succeed {
        let body = support::unwrap_body_to_string(&response).unwrap();
        let ticket_response: R = serde_json::from_str(&body).unwrap();
        assert_eq!(ticket_response.success, false);
        //Now try with redeem code being correct
        let request_data = TicketRedeemRequest {
            redeem_key: ticket.redeem_key.unwrap(),
        };

        let response = tickets::redeem((
            database.connection.clone().into(),
            path2,
            Json(request_data),
            auth_user,
            request.extract_state(),
        )).unwrap();
        let body = support::unwrap_body_to_string(&response).unwrap();
        let ticket_response: R = serde_json::from_str(&body).unwrap();
        assert_eq!(ticket_response.success, true);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn show_redeemable_ticket(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let request = TestRequest::create();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .with_venue(&venue)
        .finish();
    let user2 = database.create_user().finish();
    let mut cart = Order::create(user2.id, OrderTypes::Cart)
        .commit(&database.connection)
        .unwrap();
    let ticket_type = &event.ticket_types(&database.connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 1, &database.connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(&database.connection).unwrap();
    cart.add_external_payment("test".to_string(), user.id, total, &database.connection)
        .unwrap();

    let ticket = TicketInstance::find(ticket.id, &database.connection).unwrap();

    let mut path = Path::<PathParameters>::extract(&request.request).unwrap();
    path.id = ticket.id;

    let response = tickets::show_redeemable_ticket((
        database.connection.clone().into(),
        path,
        auth_user.clone(),
    )).unwrap();

    if should_test_succeed {
        let body = support::unwrap_body_to_string(&response).unwrap();
        let ticket_response: RedeemableTicket = serde_json::from_str(&body).unwrap();
        assert!(ticket_response.redeem_key.is_some());
    } else {
        support::expects_unauthorized(&response);
    }
}
