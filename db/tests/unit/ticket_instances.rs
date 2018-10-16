use bigneon_db::models::{
    DisplayTicket, DisplayUser, EventEditableAttributes, Order, OrderTypes, RedeemResults,
    TicketInstance, Wallet,
};
use chrono::prelude::*;
use chrono::NaiveDateTime;
use diesel;
use diesel::result::Error;
use diesel::sql_types;
use diesel::Connection;
use diesel::RunQueryDsl;
use support::project::TestProject;
use time::Duration;
use uuid::Uuid;

#[test]
pub fn find_for_user_for_display() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_event_start(&NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let event2 = project
        .create_event()
        .with_organization(&organization)
        .with_event_start(&NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 10, 11))
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let ticket_type2 = &event2.ticket_types(connection).unwrap()[0];
    let mut ticket_ids: Vec<Uuid> = cart
        .add_tickets(ticket_type.id, 2, connection)
        .unwrap()
        .into_iter()
        .map(|t| t.id)
        .collect();
    ticket_ids.sort();
    let mut ticket_ids2: Vec<Uuid> = cart
        .add_tickets(ticket_type2.id, 2, connection)
        .unwrap()
        .into_iter()
        .map(|t| t.id)
        .collect();
    ticket_ids2.sort();

    // Order is not paid so tickets are not accessible
    assert!(
        TicketInstance::find_for_user_for_display(user.id, Some(event.id), None, None, connection)
            .unwrap()
            .is_empty()
    );

    // Order is paid, tickets returned
    let total = cart.calculate_total(connection).unwrap();
    cart.add_external_payment("test".to_string(), user.id, total, connection)
        .unwrap();

    let mut found_ticket_ids: Vec<Uuid> =
        TicketInstance::find_for_user_for_display(user.id, Some(event.id), None, None, connection)
            .unwrap()
            .iter()
            .flat_map(move |(_, tickets)| tickets.iter())
            .map(|t| t.id)
            .collect();
    found_ticket_ids.sort();
    assert_eq!(found_ticket_ids, ticket_ids);

    // other event
    let mut found_ticket_ids: Vec<Uuid> =
        TicketInstance::find_for_user_for_display(user.id, Some(event2.id), None, None, connection)
            .unwrap()
            .iter()
            .flat_map(move |(_, tickets)| tickets.iter())
            .map(|t| t.id)
            .collect();
    found_ticket_ids.sort();
    assert_eq!(found_ticket_ids, ticket_ids2);

    // no event specified
    let mut all_ticket_ids = ticket_ids.clone();
    all_ticket_ids.append(&mut ticket_ids2.clone());
    all_ticket_ids.sort();
    let mut found_ticket_ids: Vec<Uuid> =
        TicketInstance::find_for_user_for_display(user.id, None, None, None, connection)
            .unwrap()
            .iter()
            .flat_map(move |(_, tickets)| tickets.iter())
            .map(|t| t.id)
            .collect();
    found_ticket_ids.sort();
    assert_eq!(found_ticket_ids, all_ticket_ids);

    // start date prior to both event starts
    let mut found_ticket_ids: Vec<Uuid> = TicketInstance::find_for_user_for_display(
        user.id,
        None,
        Some(NaiveDate::from_ymd(2015, 7, 8).and_hms(9, 0, 11)),
        None,
        connection,
    ).unwrap()
    .iter()
    .flat_map(move |(_, tickets)| tickets.iter())
    .map(|t| t.id)
    .collect();
    found_ticket_ids.sort();
    assert_eq!(found_ticket_ids, all_ticket_ids);

    // start date filters out event
    let mut found_ticket_ids: Vec<Uuid> = TicketInstance::find_for_user_for_display(
        user.id,
        None,
        Some(NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 0, 11)),
        None,
        connection,
    ).unwrap()
    .iter()
    .flat_map(move |(_, tickets)| tickets.iter())
    .map(|t| t.id)
    .collect();
    found_ticket_ids.sort();
    assert_eq!(found_ticket_ids, ticket_ids2);

    // end date filters out event
    let mut found_ticket_ids: Vec<Uuid> = TicketInstance::find_for_user_for_display(
        user.id,
        None,
        None,
        Some(NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 0, 11)),
        connection,
    ).unwrap()
    .iter()
    .flat_map(move |(_, tickets)| tickets.iter())
    .map(|t| t.id)
    .collect();
    found_ticket_ids.sort();
    assert_eq!(found_ticket_ids, ticket_ids);
}

#[test]
pub fn find() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user: DisplayUser = project.create_user().finish().into();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let display_event = event.for_display(connection).unwrap();
    let ticket = cart
        .add_tickets(ticket_type.id, 1, connection)
        .unwrap()
        .remove(0);
    let expected_ticket = DisplayTicket {
        id: ticket.id,
        ticket_type_name: ticket_type.name.clone(),
    };
    assert_eq!(
        (display_event, user, expected_ticket),
        TicketInstance::find_for_display(ticket.id, connection).unwrap()
    );
    assert!(TicketInstance::find(Uuid::new_v4(), connection).is_err());
}

#[test]
pub fn find_for_user() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = project.create_user().finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 5, connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(connection).unwrap();
    cart.add_external_payment("test".to_string(), user.id, total, connection)
        .unwrap();

    let tickets = TicketInstance::find_for_user(user.id, connection).unwrap();

    assert_eq!(tickets.len(), 5);
    assert!(TicketInstance::find(Uuid::new_v4(), connection).is_err());
}

#[test]
pub fn reserve_tickets() {
    let db = TestProject::new();
    let connection = db.get_connection();

    let organization = db
        .create_organization()
        .with_fee_schedule(&db.create_fee_schedule().finish())
        .finish();
    let event = db
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let user = db.create_user().finish();
    let order = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type_id = event.ticket_types(connection).unwrap()[0].id;
    let expires = NaiveDateTime::from(Utc::now().naive_utc() + Duration::days(2));
    order.add_tickets(ticket_type_id, 0, connection).unwrap();
    let order_item = order.items(connection).unwrap().remove(0);

    let reserved_tickets = TicketInstance::reserve_tickets(
        &order_item,
        &expires,
        ticket_type_id,
        None,
        10,
        connection,
    ).unwrap();
    let order_item = order.items(connection).unwrap().remove(0);
    assert_eq!(reserved_tickets.len(), 10);

    assert!(
        reserved_tickets
            .iter()
            .filter(|&ticket| ticket.order_item_id != Some(order_item.id))
            .collect::<Vec<&TicketInstance>>()
            .is_empty()
    );
    assert!(
        reserved_tickets
            .iter()
            .filter(|&ticket| ticket.reserved_until.is_none())
            .collect::<Vec<&TicketInstance>>()
            .is_empty()
    );
}

#[test]
pub fn release_tickets() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project.create_event().with_ticket_pricing().finish();
    let user = project.create_user().finish();
    let order = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type_id = event.ticket_types(connection).unwrap()[0].id;
    order.add_tickets(ticket_type_id, 10, connection).unwrap();
    let order_item = order.items(connection).unwrap().remove(0);

    // Release tickets
    let released_tickets =
        TicketInstance::release_tickets(&order_item, Some(4), connection).unwrap();

    assert_eq!(released_tickets.len(), 4);
    assert!(
        released_tickets
            .iter()
            .filter(|&ticket| ticket.order_item_id == Some(order_item.id))
            .collect::<Vec<&TicketInstance>>()
            .is_empty()
    );
    assert!(
        released_tickets
            .iter()
            .filter(|&ticket| ticket.reserved_until.is_some())
            .collect::<Vec<&TicketInstance>>()
            .is_empty()
    );

    project
        .get_connection()
        .transaction::<Vec<TicketInstance>, Error, _>(|| {
            // Release requesting too many tickets
            let released_tickets =
                TicketInstance::release_tickets(&order_item, Some(7), connection);
            assert_eq!(
                released_tickets.unwrap_err().cause.unwrap(),
                "Could not release the correct amount of tickets",
            );

            Err(Error::RollbackTransaction)
        }).unwrap_err();

    // Release remaining tickets (no quantity specified)
    let released_tickets = TicketInstance::release_tickets(&order_item, None, connection).unwrap();
    assert_eq!(released_tickets.len(), 6);
    assert!(
        released_tickets
            .iter()
            .filter(|&ticket| ticket.order_item_id.is_some())
            .collect::<Vec<&TicketInstance>>()
            .is_empty()
    );
    assert!(
        released_tickets
            .iter()
            .filter(|&ticket| ticket.reserved_until.is_some())
            .collect::<Vec<&TicketInstance>>()
            .is_empty()
    );
}

#[test]
fn redeem_ticket() {
    let project = TestProject::new();
    let connection = project.get_connection();

    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 1, connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(connection).unwrap();
    cart.add_external_payment("test".to_string(), user.id, total, connection)
        .unwrap();

    let ticket = TicketInstance::find(ticket.id, connection).unwrap();

    let result1 =
        TicketInstance::redeem_ticket(ticket.id, "WrongKey".to_string(), connection).unwrap();
    assert_eq!(result1, RedeemResults::TicketInvalid);
    let result2 =
        TicketInstance::redeem_ticket(ticket.id, ticket.redeem_key.unwrap(), connection).unwrap();
    assert_eq!(result2, RedeemResults::TicketRedeemSuccess);
}

#[test]
fn show_redeemable_ticket() {
    let project = TestProject::new();
    let connection = project.get_connection();

    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let venue = project.create_venue().finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .with_venue(&venue)
        .finish();
    let user = project.create_user().finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 1, connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(connection).unwrap();
    cart.add_external_payment("test".to_string(), user.id, total, connection)
        .unwrap();

    let ticket = TicketInstance::find(ticket.id, connection).unwrap();

    //make redeem date in the future
    let new_event_redeem_date = EventEditableAttributes {
        redeem_date: Some(NaiveDateTime::from(
            Utc::now().naive_utc() + Duration::days(2),
        )),
        ..Default::default()
    };

    event.update(new_event_redeem_date, connection).unwrap();

    let result = TicketInstance::show_redeemable_ticket(ticket.id, connection).unwrap();
    assert!(result.redeem_key.is_none());

    //make redeem date in the past
    let new_event_redeem_date = EventEditableAttributes {
        redeem_date: Some(NaiveDateTime::from(
            Utc::now().naive_utc() - Duration::days(2),
        )),
        ..Default::default()
    };

    event.update(new_event_redeem_date, connection).unwrap();

    let result = TicketInstance::show_redeemable_ticket(ticket.id, connection).unwrap();
    assert!(result.redeem_key.is_some());
}

#[test]
pub fn authorize_ticket_transfer() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = project.create_user().finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 5, connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(connection).unwrap();

    cart.add_external_payment("test".to_string(), user.id, total, connection)
        .unwrap();

    let tickets = TicketInstance::find_for_user(user.id, connection).unwrap();

    assert_eq!(tickets.len(), 5);
    //try with a ticket that does not exist in the list

    let tickets = TicketInstance::find_for_user(user.id, connection).unwrap();

    let mut ticket_ids: Vec<Uuid> = tickets.iter().map(|t| t.id).collect();
    ticket_ids.push(Uuid::new_v4());

    let transfer_auth2 =
        TicketInstance::authorize_ticket_transfer(user.id, ticket_ids, 24, connection);

    assert!(transfer_auth2.is_err());

    //Now try with tickets that the user does own

    let ticket_ids: Vec<Uuid> = tickets.iter().map(|t| t.id).collect();

    let transfer_auth3 =
        TicketInstance::authorize_ticket_transfer(user.id, ticket_ids, 24, connection).unwrap();

    assert_eq!(transfer_auth3.sender_user_id, user.id);
}

#[test]
pub fn receive_ticket_transfer() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = project.create_user().finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let ticket = cart
        .add_tickets(ticket_type.id, 5, connection)
        .unwrap()
        .remove(0);
    let total = cart.calculate_total(connection).unwrap();

    cart.add_external_payment("test".to_string(), user.id, total, connection)
        .unwrap();
    let tickets = TicketInstance::find_for_user(user.id, connection).unwrap();
    let ticket_ids: Vec<Uuid> = tickets.iter().map(|t| t.id).collect();

    let user2 = project.create_user().finish();
    //try receive ones that are expired
    let transfer_auth =
        TicketInstance::authorize_ticket_transfer(user.id, ticket_ids.clone(), 0, connection)
            .unwrap();

    let q: Vec<TicketInstance> = diesel::sql_query(
        r#"
        UPDATE ticket_instances
        SET transfer_expiry_date = '2018-06-06 09:49:09.643207'
        WHERE id = $1;
        "#,
    ).bind::<sql_types::Uuid, _>(ticket_ids[0])
    .get_results(connection)
    .unwrap();

    let receive_auth2 =
        TicketInstance::receive_ticket_transfer(transfer_auth, user2.id, connection);

    assert!(receive_auth2.is_err());

    //try receive the wrong number of tickets (too few)
    let transfer_auth =
        TicketInstance::authorize_ticket_transfer(user.id, ticket_ids.clone(), 3600, connection)
            .unwrap();

    let mut wrong_auth = transfer_auth.clone();
    wrong_auth.num_tickets = 4;
    let receive_auth1 = TicketInstance::receive_ticket_transfer(wrong_auth, user2.id, connection);
    assert!(receive_auth1.is_err());

    //legit receive tickets
    let _receive_auth3 =
        TicketInstance::receive_ticket_transfer(transfer_auth, user2.id, connection);

    //Look if one of the tickets does have the new wallet_id
    let receive_wallet = Wallet::find_default_for_user(user2.id, connection).unwrap();

    let received_ticket = TicketInstance::find(ticket_ids[0], connection).unwrap();

    assert_eq!(receive_wallet.id, received_ticket.wallet_id);
}
