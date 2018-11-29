use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::schema::order_items;
use bigneon_db::utils::errors::ErrorCode::ValidationError;
use diesel::prelude::*;

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
        false,
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
fn update_with_validation_errors() {
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
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 4,
            redemption_code: None,
        }],
        false,
        connection,
    ).unwrap();
    let code = project
        .create_code()
        .with_event(&event)
        .for_ticket_type(&ticket_type)
        .with_max_tickets_per_user(Some(5))
        .finish();

    let items = cart.items(&connection).unwrap();
    let order_item = items
        .iter()
        .find(|i| i.ticket_type_id == Some(ticket_type.id))
        .unwrap();
    diesel::update(order_items::table.filter(order_items::id.eq(order_item.id)))
        .set(order_items::code_id.eq(code.id))
        .get_result::<OrderItem>(connection)
        .unwrap();
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 5,
            redemption_code: None,
        }],
        false,
        connection,
    ).unwrap();

    let result = cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 6,
            redemption_code: None,
        }],
        false,
        connection,
    );

    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("quantity"));
                assert_eq!(errors["quantity"].len(), 1);
                assert_eq!(errors["quantity"][0].code, "max_tickets_per_user_reached");
                assert_eq!(
                    &errors["quantity"][0].message.clone().unwrap().into_owned(),
                    "Redemption code maximum tickets limit exceeded"
                );
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Different user can still add them to their cart
    let user = project.create_user().finish();
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 4,
            redemption_code: None,
        }],
        false,
        connection,
    ).unwrap();
    let order_item = cart.items(connection).unwrap().remove(0);
    diesel::update(order_items::table.filter(order_items::id.eq(order_item.id)))
        .set(order_items::code_id.eq(code.id))
        .get_result::<OrderItem>(connection)
        .unwrap();
    // Add 1 making it 5 tickets for this type
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 1,
            redemption_code: None,
        }],
        false,
        connection,
    ).unwrap();
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
        false,
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
