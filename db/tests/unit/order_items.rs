use bigneon_db::dev::TestProject;
use bigneon_db::models::*;

#[test]
fn find_fee_item() {
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
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    let ticket = &event.ticket_types(connection).unwrap()[0];
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket.id,
            quantity: 10,
            redemption_code: None,
        }],
        connection,
    ).unwrap();

    let items = cart.items(&connection).unwrap();
    let order_item = items
        .iter()
        .find(|i| i.ticket_type_id == Some(ticket.id))
        .unwrap();

    let fee_item = order_item.find_fee_item(connection).unwrap().unwrap();

    assert_eq!(fee_item.parent_id, Some(order_item.id));
    assert_eq!(fee_item.item_type, OrderItemTypes::PerUnitFees.to_string());
}

#[test]
fn calculate_quantity() {
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
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    let ticket = &event.ticket_types(connection).unwrap()[0];
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket.id,
            quantity: 10,
            redemption_code: None,
        }],
        connection,
    ).unwrap();

    let items = cart.items(&connection).unwrap();
    let order_item = items
        .iter()
        .find(|i| i.ticket_type_id == Some(ticket.id))
        .unwrap();

    assert_eq!(order_item.calculate_quantity(connection), Ok(10));

    //let datetime_in_past = NaiveDate::from_ymd(2018, 9, 16).and_hms(12, 12, 12);
    //diesel::update(ticket_instances::table.filter(ticket_instances::id.eq(tickets[0].id)))
    //    .set(ticket_instances::reserved_until.eq(datetime_in_past))
    //    .get_result::<TicketInstance>(connection)
    //    .unwrap();

    //assert_eq!(order_item.calculate_quantity(connection), Ok(9));
}
