use bigneon_api::models::UserDisplayTicketType;
use bigneon_db::models::TicketTypeStatus;
use support::database::TestDatabase;

#[test]
fn from_ticket_type() {
    let database = TestDatabase::new();
    let event = database.create_event().with_ticket_pricing().finish();
    let ticket_type = event.ticket_types(&database.connection).unwrap().remove(0);
    let ticket_pricing = ticket_type
        .current_ticket_pricing(&database.connection)
        .unwrap();

    // New event nothing sold
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &database.connection).unwrap();
    assert_eq!(display_ticket_type.quantity, 100);
    assert_eq!(
        display_ticket_type.status,
        TicketTypeStatus::Published.to_string()
    );
    assert_eq!(
        Some(ticket_pricing.into()),
        display_ticket_type.ticket_pricing,
    );

    // 10 tickets sold / reserved (via create_order for_event)
    let order = database.create_order().for_event(&event).finish();
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &database.connection).unwrap();
    assert_eq!(display_ticket_type.quantity, 90);
    assert_eq!(
        display_ticket_type.status,
        TicketTypeStatus::Published.to_string()
    );

    // Remaining tickets sold
    order
        .add_tickets(ticket_type.id, 90, &database.connection)
        .unwrap();
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &database.connection).unwrap();
    assert_eq!(display_ticket_type.quantity, 0);
    assert_eq!(
        display_ticket_type.status,
        TicketTypeStatus::SoldOut.to_string()
    );

    // Release some tickets
    let order_item = order.items(&database.connection).unwrap().remove(0);
    assert!(
        order
            .remove_tickets(order_item, Some(10), &database.connection)
            .is_ok()
    );
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &database.connection).unwrap();
    assert_eq!(display_ticket_type.quantity, 10);
    assert_eq!(
        display_ticket_type.status,
        TicketTypeStatus::Published.to_string()
    );

    // No active ticket pricing
    let event = database.create_event().with_tickets().finish();
    let ticket_type = event.ticket_types(&database.connection).unwrap().remove(0);
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &database.connection).unwrap();
    assert_eq!(display_ticket_type.quantity, 100);
    assert_eq!(
        display_ticket_type.status,
        TicketTypeStatus::NoActivePricing.to_string()
    );
}
