use actix_web::{http::StatusCode, FromRequest, Path};
use bigneon_api::controllers::tickets::{self, ShowTicketResponse};
use bigneon_api::models::{OptionalPathParameters, PathParameters};
use bigneon_db::models::*;
use functional::base;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
pub fn index() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let request = TestRequest::create();
    let organization = database.create_organization().finish();
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_name("Event1".into())
        .with_venue(&venue)
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();

    // Other event
    let event2 = database
        .create_event()
        .with_name("Event2".into())
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();

    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&database.connection)
        .unwrap();
    let ticket_type = &event.ticket_types(&database.connection).unwrap()[0];
    let ticket_type2 = &event2.ticket_types(&database.connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 1, &database.connection)
        .unwrap()
        .remove(0);
    let ticket2 = cart
        .add_tickets(ticket_type2.id, 1, &database.connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(&database.connection).unwrap();
    cart.add_external_payment("test".to_string(), user.id, total, &database.connection)
        .unwrap();

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);

    // Test with specified event
    let mut path = Path::<OptionalPathParameters>::extract(&request.request).unwrap();
    path.id = Some(event.id);
    let response =
        tickets::index((database.connection.clone().into(), path, auth_user.clone())).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let found_tickets: Vec<DisplayTicket> = serde_json::from_str(&body).unwrap();
    let expected_ticket = DisplayTicket {
        id: ticket.id,
        ticket_type_name: ticket_type.name.clone(),
    };
    assert_eq!(vec![expected_ticket.clone()], found_tickets);

    // Test without specified event
    let mut path = Path::<OptionalPathParameters>::extract(&request.request).unwrap();
    path.id = None;
    let response = tickets::index((database.connection.clone().into(), path, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let found_tickets: Vec<(DisplayEvent, Vec<DisplayTicket>)> =
        serde_json::from_str(&body).unwrap();
    let expected_ticket2 = DisplayTicket {
        id: ticket2.id,
        ticket_type_name: ticket_type2.name.clone(),
    };
    assert_eq!(
        vec![
            (
                event.for_display(&database.connection).unwrap(),
                vec![expected_ticket]
            ),
            (
                event2.for_display(&database.connection).unwrap(),
                vec![expected_ticket2]
            )
        ],
        found_tickets
    );
}

#[test]
pub fn show() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let request = TestRequest::create();
    let organization = database.create_organization().finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
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
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let mut path = Path::<PathParameters>::extract(&request.request).unwrap();
    path.id = ticket.id;
    let response = tickets::show((database.connection.clone().into(), path, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let ticket_response: ShowTicketResponse = serde_json::from_str(&body).unwrap();
    let expected_ticket = DisplayTicket {
        id: ticket.id,
        ticket_type_name: ticket_type.name.clone(),
    };

    let expected_result = ShowTicketResponse {
        ticket: expected_ticket,
        user: user.into(),
        event: event.for_display(&database.connection).unwrap(),
    };
    assert_eq!(expected_result, ticket_response);
}

#[cfg(test)]
mod show_other_user_ticket_tests {
    use super::*;
    #[test]
    fn show_other_user_ticket_org_member() {
        base::tickets::show_other_user_ticket(Roles::OrgMember, true, true);
    }
    #[test]
    fn show_other_user_ticket_admin() {
        base::tickets::show_other_user_ticket(Roles::Admin, true, true);
    }
    #[test]
    fn show_other_user_ticket_user() {
        base::tickets::show_other_user_ticket(Roles::User, false, true);
    }
    #[test]
    fn show_other_user_ticket_org_owner() {
        base::tickets::show_other_user_ticket(Roles::OrgOwner, true, true);
    }
    #[test]
    fn show_other_user_ticket_other_organization_org_member() {
        base::tickets::show_other_user_ticket(Roles::OrgMember, false, false);
    }
    #[test]
    fn show_other_user_ticket_other_organization_admin() {
        base::tickets::show_other_user_ticket(Roles::Admin, true, false);
    }
    #[test]
    fn show_other_user_ticket_other_organization_user() {
        base::tickets::show_other_user_ticket(Roles::User, false, false);
    }
    #[test]
    fn show_other_user_ticket_other_organization_org_owner() {
        base::tickets::show_other_user_ticket(Roles::OrgOwner, false, false);
    }
}
