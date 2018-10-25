use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::schema::orders;
use bigneon_db::utils::errors::ErrorCode::ValidationError;
use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::Connection;
use time::Duration;
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
    let order = Order::find_or_create_cart(&user, project.get_connection()).unwrap();
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
    let cart = Order::find_or_create_cart(&user, connection).unwrap();
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

    // Add some more
    let tickets = cart.add_tickets(ticket.id, 5, connection).unwrap();
    assert_eq!(tickets.len(), 5);
    let items = cart.items(connection).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].calculate_quantity(connection), Ok(15));
}

#[test]
fn add_tickets_with_increment() {
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
    let cart = Order::find_or_create_cart(&user, connection).unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let update_parameters = TicketTypeEditableAttributes {
        increment: Some(4),
        ..Default::default()
    };
    let ticket_type = ticket_type.update(update_parameters, connection).unwrap();

    let add_tickets_result = cart.add_tickets(ticket_type.id, 10, connection);
    assert!(add_tickets_result.is_err());
    let error = add_tickets_result.unwrap_err();
    match &error.error_code {
        ValidationError { errors } => {
            assert!(errors.contains_key("quantity"));
            assert_eq!(errors["quantity"].len(), 1);
            assert_eq!(errors["quantity"][0].code, "quantity_invalid_increment");
        }
        _ => panic!("Expected validation error"),
    }

    let add_tickets_result = cart.add_tickets(ticket_type.id, 4, connection);
    assert!(add_tickets_result.is_ok());

    let add_tickets_result = cart.add_tickets(ticket_type.id, 8, connection);
    assert!(add_tickets_result.is_ok());
    assert_eq!(cart.items(connection).unwrap()[0].quantity, 12);
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
    let cart = Order::find_or_create_cart(&user, connection).unwrap();
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
    assert!(
        cart.remove_tickets(ticket_pricing.id, Some(4), connection)
            .is_ok()
    );
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
            let removed_tickets = cart.remove_tickets(ticket_pricing.id, Some(7), connection);
            assert_eq!(
                removed_tickets.unwrap_err().cause.unwrap(),
                "Could not release the correct amount of tickets",
            );

            Err(Error::RollbackTransaction)
        }).unwrap_err();

    // Release remaining tickets (no quantity specified so removes remaining)
    assert!(
        cart.remove_tickets(ticket_pricing.id, None, connection)
            .is_ok()
    );
    // Item removed from cart completely
    assert!(cart.items(connection).unwrap().is_empty());
}

#[test]
fn remove_tickets_with_increment() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    let cart = Order::find_or_create_cart(&user, connection).unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    let update_parameters = TicketTypeEditableAttributes {
        increment: Some(4),
        ..Default::default()
    };
    let ticket_type = ticket_type.update(update_parameters, connection).unwrap();
    let add_tickets_result = cart.add_tickets(ticket_type.id, 8, connection);
    assert!(add_tickets_result.is_ok());
    assert_eq!(cart.items(connection).unwrap()[0].quantity, 8);
    let order_item = cart.items(connection).unwrap().remove(0);

    assert!(
        cart.remove_tickets(order_item.ticket_pricing_id.unwrap(), Some(4), connection)
            .is_ok()
    );
    let order_item = cart.items(connection).unwrap().remove(0);
    assert_eq!(order_item.quantity, 4);

    let remove_tickets_result =
        cart.remove_tickets(order_item.ticket_pricing_id.unwrap(), Some(3), connection);
    assert!(remove_tickets_result.is_err());
    let error = remove_tickets_result.unwrap_err();
    match &error.error_code {
        ValidationError { errors } => {
            assert!(errors.contains_key("quantity"));
            assert_eq!(errors["quantity"].len(), 1);
            assert_eq!(errors["quantity"][0].code, "quantity_invalid_increment");
        }
        _ => panic!("Expected validation error"),
    }
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
    let order = Order::find_or_create_cart(&user, connection).unwrap();
    order.add_tickets(ticket_type.id, 10, connection).unwrap();
    let order2 = Order::find_or_create_cart(&user2, connection).unwrap();
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
fn find_cart_for_user() {
    let project = TestProject::new();
    let user = project.create_user().finish();

    // No cart
    let conn = project.get_connection();
    let cart_result = Order::find_cart_for_user(user.id, conn).unwrap();
    assert!(cart_result.is_none());

    // Cart exists, is not expired
    let cart = Order::find_or_create_cart(&user, conn).unwrap();
    let cart_result = Order::find_cart_for_user(user.id, conn);
    assert_eq!(cart_result.unwrap().unwrap(), cart);

    // Expired cart
    let one_minute_ago = NaiveDateTime::from(Utc::now().naive_utc() - Duration::minutes(1));
    diesel::update(&cart)
        .set(orders::expires_at.eq(one_minute_ago))
        .get_result::<Order>(conn)
        .unwrap();
    let cart_result = Order::find_cart_for_user(user.id, conn).unwrap();
    assert!(cart_result.is_none());
}

#[test]
fn has_items() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let cart = Order::find_or_create_cart(&user, connection).unwrap();

    // Without items
    assert!(!cart.has_items(connection).unwrap());

    // With items
    let event = project
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    cart.add_tickets(ticket_type.id, 10, connection).unwrap();
    assert!(cart.has_items(connection).unwrap());
}

#[test]
fn destroy() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let cart = Order::find_or_create_cart(&user, connection).unwrap();
    let cart_result = Order::find_cart_for_user(user.id, connection).unwrap();
    assert_eq!(cart_result.unwrap(), cart);

    cart.destroy(connection).unwrap();
    let cart_result = Order::find_cart_for_user(user.id, connection).unwrap();
    assert!(cart_result.is_none());
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
    let conn = project.get_connection();
    let cart = Order::find_or_create_cart(&user, conn).unwrap();
    let ticket = &event.ticket_types(conn).unwrap()[0];
    cart.add_tickets(ticket.id, 10, conn).unwrap();

    let total = cart.calculate_total(conn).unwrap();
    assert_eq!(total, 1700);

    cart.add_tickets(ticket.id, 20, conn).unwrap();
    let total = cart.calculate_total(conn).unwrap();
    assert_eq!(total, 5100);
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
    let conn = project.get_connection();
    let mut cart = Order::find_or_create_cart(&user, conn).unwrap();
    let ticket = &event.ticket_types(conn).unwrap()[0];
    cart.add_tickets(ticket.id, 10, conn).unwrap();
    assert_eq!(cart.calculate_total(conn).unwrap(), 2000);
    cart.add_external_payment("test".to_string(), user.id, 1500, conn)
        .unwrap();
    assert_eq!(cart.status(), OrderStatus::PartiallyPaid);
    cart.add_external_payment("test2".to_string(), user.id, 500, conn)
        .unwrap();
    assert_eq!(cart.status(), OrderStatus::Paid);
}

#[test]
fn find_for_user_for_display() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let mut order1 = project.create_order().for_user(&user).finish();
    order1
        .add_external_payment("test".to_string(), user.id, 2000, project.get_connection())
        .unwrap();
    let mut order2 = project.create_order().for_user(&user).finish();
    order2
        .add_external_payment("test".to_string(), user.id, 500, project.get_connection())
        .unwrap();

    assert_eq!(order1.status, OrderStatus::Paid.to_string());
    assert_eq!(order2.status, OrderStatus::PartiallyPaid.to_string());

    let display_orders =
        Order::find_for_user_for_display(user.id, project.get_connection()).unwrap();
    let ids: Vec<Uuid> = display_orders.iter().map(|o| o.id).collect();
    //The order of the ids is not certain so this test fails from time to time.
    //It is ordered by updated_at which is the same for the two orders

    assert!(
        (order1.id == ids[0] && order2.id == ids[1])
            || (order1.id == ids[1] && order2.id == ids[0])
    );
}

#[test]
fn for_display() {
    let project = TestProject::new();
    let order = project.create_order().finish();

    // 1 minute from now expires
    let one_minute_from_now = NaiveDateTime::from(Utc::now().naive_utc() + Duration::minutes(1));
    let order = diesel::update(orders::table.filter(orders::id.eq(order.id)))
        .set(orders::expires_at.eq(one_minute_from_now))
        .get_result::<Order>(project.get_connection())
        .unwrap();
    let display_order = order.for_display(project.get_connection()).unwrap();
    assert!(display_order.seconds_until_expiry <= 60 && display_order.seconds_until_expiry >= 59);

    // 1 minute ago expires
    let one_minute_ago = NaiveDateTime::from(Utc::now().naive_utc() - Duration::minutes(1));
    let order = diesel::update(orders::table.filter(orders::id.eq(order.id)))
        .set(orders::expires_at.eq(one_minute_ago))
        .get_result::<Order>(project.get_connection())
        .unwrap();
    let display_order = order.for_display(project.get_connection()).unwrap();
    assert_eq!(0, display_order.seconds_until_expiry);
}

#[test]
fn adding_event_fees() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let event1 = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .with_event_fee()
        .finish();
    let event2 = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .with_event_fee()
        .finish();
    let event3 = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let organization2 = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .with_event_fee()
        .finish();
    let event4 = project
        .create_event()
        .with_organization(&organization2)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    let cart = Order::find_or_create_cart(&user, connection).unwrap();
    let ticket1 = &event1.ticket_types(connection).unwrap()[0];
    let ticket2 = &event2.ticket_types(connection).unwrap()[0];
    let ticket3 = &event3.ticket_types(connection).unwrap()[0];
    let ticket4 = &event4.ticket_types(connection).unwrap()[0];
    let _tickets = cart.add_tickets(ticket1.id, 10, connection).unwrap();
    cart.update_event_fees(connection).unwrap();

    let order_items = OrderItem::find_for_order(cart.id, connection).unwrap();

    let mut event_fees_count = 0;
    for o in &order_items {
        if o.item_type == OrderItemTypes::EventFees.to_string() {
            event_fees_count += 1;
        }
    }
    assert_eq!(event_fees_count, 1);

    //Add some more of the same event and some of a second event
    let _tickets = cart.add_tickets(ticket1.id, 5, connection).unwrap();
    let _tickets = cart.add_tickets(ticket2.id, 5, connection).unwrap();
    cart.update_event_fees(connection).unwrap();
    let order_items = OrderItem::find_for_order(cart.id, connection).unwrap();

    let mut event_fees_count = 0;
    for o in &order_items {
        if o.item_type == OrderItemTypes::EventFees.to_string() {
            event_fees_count += 1;
        }
    }
    assert_eq!(event_fees_count, 2);

    //Add tickets with null event fee and null organization event_fee

    let _tickets = cart.add_tickets(ticket3.id, 5, connection).unwrap();
    cart.update_event_fees(connection).unwrap();
    let order_items = OrderItem::find_for_order(cart.id, connection).unwrap();

    let mut event_fees_count = 0;
    for o in &order_items {
        if o.item_type == OrderItemTypes::EventFees.to_string() {
            event_fees_count += 1;
        }
    }
    assert_eq!(event_fees_count, 2);

    //Add tickets with null event fee and but default organization event_fee

    let _tickets = cart.add_tickets(ticket4.id, 5, connection).unwrap();
    cart.update_event_fees(connection).unwrap();
    let order_items = OrderItem::find_for_order(cart.id, connection).unwrap();

    let mut event_fees_count = 0;
    for o in &order_items {
        if o.item_type == OrderItemTypes::EventFees.to_string() {
            event_fees_count += 1;
        }
    }
    assert_eq!(event_fees_count, 3);
}
