use bigneon_db::models::{Order, OrderTypes, TicketInstance};
use chrono::prelude::*;
use chrono::NaiveDateTime;
use diesel::result::Error;
use diesel::Connection;
use support::project::TestProject;
use time::Duration;

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
