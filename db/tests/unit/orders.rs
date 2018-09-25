use bigneon_db::models::*;
use diesel::result::Error;
use diesel::Connection;
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let venue = project.create_venue().finish();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let _event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let order = Order::create(user.id, OrderTypes::Cart)
        .commit(project.get_connection())
        .unwrap();
    assert_eq!(order.user_id, user.id);
    assert_eq!(order.id.to_string().is_empty(), false);
}

#[test]
fn add_tickets() {
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
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket = &event.ticket_types(connection).unwrap()[0];
    let tickets = cart.add_tickets(ticket.id, 10, connection).unwrap();
    assert_eq!(tickets.len(), 10);
    let order_item = cart.items(connection).unwrap().remove(0);
    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), connection).unwrap();
    assert_eq!(order_item.calculate_quantity(connection), Ok(10));
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );

    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), connection).unwrap();
    let fee_item = order_item.find_fee_item(connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 10
    );
}

#[test]
fn remove_tickets() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let tickets = cart.add_tickets(ticket_type.id, 10, connection).unwrap();
    assert_eq!(tickets.len(), 10);
    let order_item = cart.items(connection).unwrap().remove(0);
    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), connection).unwrap();
    assert_eq!(order_item.quantity, 10);
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );

    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), connection).unwrap();
    let fee_item = order_item.find_fee_item(connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 10
    );

    // Remove tickets
    assert!(cart.remove_tickets(order_item, Some(4), connection).is_ok());
    let order_item = cart.items(connection).unwrap().remove(0);
    assert_eq!(order_item.quantity, 6);
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );
    let fee_item = order_item.find_fee_item(connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 6
    );

    project
        .get_connection()
        .transaction::<Vec<TicketInstance>, Error, _>(|| {
            // Release requesting too many tickets
            let removed_tickets = cart.remove_tickets(order_item.clone(), Some(7), connection);
            assert_eq!(
                removed_tickets.unwrap_err().cause.unwrap(),
                "Could not release the correct amount of tickets",
            );

            Err(Error::RollbackTransaction)
        }).unwrap_err();

    // Release remaining tickets (no quantity specified so removes remaining)
    assert!(cart.remove_tickets(order_item, None, connection).is_ok());
    // Item removed from cart completely
    assert!(cart.items(connection).unwrap().is_empty());
}

#[test]
fn find_item() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let order = Order::create(user.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    order.add_tickets(ticket_type.id, 10, connection).unwrap();
    let order2 = Order::create(user2.id, OrderTypes::Cart)
        .commit(connection)
        .unwrap();
    order2.add_tickets(ticket_type.id, 10, connection).unwrap();

    let order_item = order.items(connection).unwrap().remove(0);
    let order_item2 = order2.items(connection).unwrap().remove(0);

    let found_item = order.find_item(order_item.id.clone(), connection).unwrap();
    assert_eq!(order_item, found_item);

    let found_item = order2
        .find_item(order_item2.id.clone(), connection)
        .unwrap();
    assert_eq!(order_item2, found_item);

    let find_results = order.find_item(order_item2.id.clone(), connection);
    assert_eq!(
        find_results.unwrap_err().cause,
        Some("Could not retrieve order item, NotFound".into())
    );

    let find_results = order2.find_item(order_item.id.clone(), connection);
    assert_eq!(
        find_results.unwrap_err().cause,
        Some("Could not retrieve order item, NotFound".into())
    );

    let find_results = order.find_item(Uuid::new_v4(), connection);
    assert_eq!(
        find_results.unwrap_err().cause,
        Some("Could not retrieve order item, NotFound".into())
    );
}

#[test]
fn find_by_user_when_cart_does_not_exist() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let cart_result = Order::find_cart_for_user(user.id, project.get_connection());
    assert_eq!(cart_result.err().unwrap().code, 2000);
}

#[test]
fn calculate_cart_total() {
    let project = TestProject::new();
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
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(project.get_connection())
        .unwrap();
    let ticket = &event.ticket_types(project.get_connection()).unwrap()[0];
    cart.add_tickets(ticket.id, 10, project.get_connection())
        .unwrap();

    let total = cart.calculate_total(project.get_connection()).unwrap();
    assert_eq!(total, 1700);
}

#[test]
fn add_external_payment() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let event = project
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(project.get_connection())
        .unwrap();
    let ticket = &event.ticket_types(project.get_connection()).unwrap()[0];
    cart.add_tickets(ticket.id, 10, project.get_connection())
        .unwrap();
    cart.add_external_payment("test".to_string(), user.id, 100, project.get_connection())
        .unwrap();
    assert_eq!(cart.status(), OrderStatus::Paid);
}
