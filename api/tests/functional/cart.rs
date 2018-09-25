use actix_web::FromRequest;
use actix_web::{http::StatusCode, HttpResponse, Json, Path};
use bigneon_api::controllers::cart;
use bigneon_api::controllers::cart::PaymentRequest;
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
fn add() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = database.create_user().finish();
    let ticket_type_id = event.ticket_types(&connection).unwrap()[0].id;

    let input = Json(cart::AddToCartRequest {
        ticket_type_id: ticket_type_id,
        quantity: 2,
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let response = cart::add((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let cart = Order::find_cart_for_user(user.id, &connection).unwrap();
    let order_item = cart.items(&connection).unwrap().remove(0);
    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), &connection).unwrap();
    assert_eq!(order_item.quantity, 2);
    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), &connection).unwrap();
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 2
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );
}

#[test]
fn add_with_existing_cart() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = database.create_user().finish();
    let ticket_type_id = event.ticket_types(&connection).unwrap()[0].id;
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();

    let input = Json(cart::AddToCartRequest {
        ticket_type_id: ticket_type_id,
        quantity: 2,
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let response = cart::add((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let order_item = cart.items(&connection).unwrap().remove(0);
    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), &connection).unwrap();
    assert_eq!(order_item.quantity, 2);
    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), &connection).unwrap();
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 2
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );
}

#[test]
fn remove() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type_id = event.ticket_types(&connection).unwrap()[0].id;
    let user = database.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();
    cart.add_tickets(ticket_type_id, 10, &connection).unwrap();

    let order_item = cart.items(&connection).unwrap().remove(0);
    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), &connection).unwrap();
    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), &connection).unwrap();
    assert_eq!(order_item.quantity, 10);
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 10
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );

    let input = Json(cart::RemoveCartRequest {
        cart_item_id: order_item.id,
        quantity: Some(4),
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let response = cart::remove((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let order_item = cart.items(&connection).unwrap().remove(0);
    assert_eq!(order_item.quantity, 6);
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 6
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );
}

#[test]
fn remove_with_no_specified_quantity() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type_id = event.ticket_types(&connection).unwrap()[0].id;
    let user = database.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();
    cart.add_tickets(ticket_type_id, 10, &connection).unwrap();
    let order_item = cart.items(&connection).unwrap().remove(0);
    let input = Json(cart::RemoveCartRequest {
        cart_item_id: order_item.id,
        quantity: None,
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let response = cart::remove((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    assert!(cart.items(&connection).unwrap().is_empty());
}

#[test]
fn remove_with_cart_item_not_belonging_to_current_cart() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type_id = event.ticket_types(&connection).unwrap()[0].id;
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);

    // Cart item belongs to user2, not user
    let user2 = database.create_user().finish();
    let cart = Order::create(user2.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();
    cart.add_tickets(ticket_type_id, 10, &connection).unwrap();
    let order_item = cart.items(&connection).unwrap().remove(0);

    let input = Json(cart::RemoveCartRequest {
        cart_item_id: order_item.id,
        quantity: None,
    });

    let response = cart::remove((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[test]
fn remove_with_no_cart() {
    let database = TestDatabase::new();
    database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user = database.create_user().finish();

    let input = Json(cart::RemoveCartRequest {
        cart_item_id: Uuid::new_v4(),
        quantity: None,
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let response = cart::remove((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[test]
fn remove_more_tickets_than_user_has() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type_id = event.ticket_types(&connection).unwrap()[0].id;
    let user = database.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();
    cart.add_tickets(ticket_type_id, 10, &connection).unwrap();

    let order_item = cart.items(&connection).unwrap().remove(0);
    let input = Json(cart::RemoveCartRequest {
        cart_item_id: order_item.id,
        quantity: Some(14),
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let response: HttpResponse =
        cart::remove((database.connection.into(), input, auth_user)).into();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn checkout_external() {
    let database = TestDatabase::new();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = database.create_user().finish();

    let order = database
        .create_cart()
        .for_user(&user)
        .for_event(&event)
        .finish();
    let request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&request.request).unwrap();
    path.id = order.id;

    let input = Json(cart::CheckoutCartRequest {
        amount: 100,
        method: PaymentRequest::External {
            reference: "TestRef".to_string(),
        },
    });

    // Must be admin to check out external
    let user = support::create_auth_user_from_user(&user, Roles::Admin, &database);

    let response = cart::checkout((database.connection.into(), input, path, user)).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
