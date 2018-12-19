use bigneon_api::models::{DisplayTicketPricing, UserDisplayTicketType};
use bigneon_db::prelude::*;
use support::database::TestDatabase;

#[test]
fn from_ticket_type() {
    let database = TestDatabase::new();

    let admin = database.create_user().finish();
    let fee_schedule = database.create_fee_schedule().finish(admin.id);
    let organization = database
        .create_organization()
        .with_fee_schedule(&fee_schedule)
        .finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
    let conn = database.connection.get();

    let ticket_type = event.ticket_types(conn).unwrap().remove(0);
    let ticket_pricing = ticket_type.current_ticket_pricing(conn).unwrap();

    // New event nothing sold
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &fee_schedule, conn).unwrap();
    assert_eq!(display_ticket_type.available, 100);
    assert_eq!(display_ticket_type.status, TicketTypeStatus::Published);
    assert_eq!(
        Some(
            DisplayTicketPricing::from_ticket_pricing(&ticket_pricing, &fee_schedule, conn)
                .unwrap()
        ),
        display_ticket_type.ticket_pricing,
    );
    assert_eq!(20, display_ticket_type.ticket_pricing.unwrap().fee_in_cents,);

    // 10 tickets sold / reserved (via create_order for_event)
    let mut order = database
        .create_order()
        .for_event(&event)
        .quantity(10)
        .finish();
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &fee_schedule, conn).unwrap();
    assert_eq!(display_ticket_type.available, 90);
    assert_eq!(display_ticket_type.status, TicketTypeStatus::Published);

    // Remaining tickets sold
    order
        .update_quantities(
            &[UpdateOrderItem {
                ticket_type_id: ticket_type.id,
                quantity: 100,
                redemption_code: None,
            }],
            false,
            conn,
        )
        .unwrap();
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &fee_schedule, conn).unwrap();
    assert_eq!(display_ticket_type.available, 0);
    assert_eq!(display_ticket_type.status, TicketTypeStatus::SoldOut);

    // Release some tickets
    order
        .update_quantities(
            &[UpdateOrderItem {
                ticket_type_id: ticket_type.id,
                quantity: 90,
                redemption_code: None,
            }],
            false,
            conn,
        )
        .unwrap();
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &fee_schedule, conn).unwrap();
    assert_eq!(display_ticket_type.available, 10);
    assert_eq!(display_ticket_type.status, TicketTypeStatus::Published);

    // No active ticket pricing
    let event = database.create_event().with_tickets().finish();
    let ticket_type = event.ticket_types(conn).unwrap().remove(0);
    let display_ticket_type =
        UserDisplayTicketType::from_ticket_type(&ticket_type, &fee_schedule, conn).unwrap();
    assert_eq!(display_ticket_type.available, 100);
    assert_eq!(
        display_ticket_type.status,
        TicketTypeStatus::NoActivePricing
    );
}
