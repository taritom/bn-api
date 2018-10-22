use actix_web::{http::StatusCode, HttpResponse, Json};
use bigneon_api::controllers::cart;
use bigneon_api::controllers::cart::{CartResponse, PaymentRequest};
use bigneon_db::models::*;
use bigneon_db::schema::orders;
use chrono::prelude::*;
use chrono::Duration;
use diesel;
use diesel::prelude::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
fn show() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let user = database.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response: HttpResponse = cart::show((database.connection.into(), auth_user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let cart_response: DisplayOrder = serde_json::from_str(&body).unwrap();
    assert_eq!(cart.id, cart_response.id);
}

#[test]
fn show_no_cart() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response: HttpResponse = cart::show((database.connection.into(), auth_user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, "{}");
}

#[test]
fn show_expired_cart() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let user = database.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();
    let one_minute_ago = NaiveDateTime::from(Utc::now().naive_utc() - Duration::minutes(1));
    diesel::update(&cart)
        .set(orders::expires_at.eq(one_minute_ago))
        .get_result::<Order>(&*connection)
        .unwrap();

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response: HttpResponse = cart::show((database.connection.into(), auth_user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, "{}");
}

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
        items: vec![cart::AddToCartRequestItem {
            ticket_type_id,
            quantity: 2,
        }],
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
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
fn add_multiple() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_type_count(2)
        .with_ticket_pricing()
        .finish();

    let user = database.create_user().finish();
    let ticket_types = event.ticket_types(&connection).unwrap();
    let ticket_type_id = ticket_types[0].id;
    let ticket_type_id2 = ticket_types[1].id;

    let input = Json(cart::AddToCartRequest {
        items: vec![
            cart::AddToCartRequestItem {
                ticket_type_id,
                quantity: 2,
            },
            cart::AddToCartRequestItem {
                ticket_type_id: ticket_type_id2,
                quantity: 3,
            },
        ],
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response = cart::add((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let cart = Order::find_cart_for_user(user.id, &connection).unwrap();
    let cart_items = cart
        .items(&connection)
        .unwrap()
        .into_iter()
        .filter(|c| c.parent_id.is_none())
        .collect::<Vec<OrderItem>>();
    let order_item = &cart_items[0];
    let order_item2 = &cart_items[1];

    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), &connection).unwrap();
    let ticket_pricing2 =
        TicketPricing::find(order_item2.ticket_pricing_id.unwrap(), &connection).unwrap();

    assert_eq!(order_item.quantity, 2);
    assert_eq!(order_item2.quantity, 3);
    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), &connection).unwrap();
    let fee_schedule_range2 =
        FeeScheduleRange::find(order_item2.fee_schedule_range_id.unwrap(), &connection).unwrap();
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    let fee_item2 = order_item2.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 2
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );
    assert_eq!(
        fee_item2.unit_price_in_cents,
        fee_schedule_range2.fee_in_cents * 3
    );
    assert_eq!(
        order_item2.unit_price_in_cents,
        ticket_pricing2.price_in_cents
    );
}

#[test]
fn add_with_increment() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = database.create_user().finish();
    let ticket_type = event.ticket_types(&connection).unwrap().remove(0);
    let update_parameters = TicketTypeEditableAttributes {
        increment: Some(4),
        ..Default::default()
    };
    let ticket_type = ticket_type.update(update_parameters, &connection).unwrap();
    let ticket_type_id = ticket_type.id;

    let input = Json(cart::AddToCartRequest {
        items: vec![cart::AddToCartRequestItem {
            ticket_type_id,
            quantity: 4,
        }],
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response = cart::add((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let cart = Order::find_cart_for_user(user.id, &connection).unwrap();
    let order_item = cart.items(&connection).unwrap().remove(0);
    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), &connection).unwrap();
    assert_eq!(order_item.quantity, 4);
    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), &connection).unwrap();
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 4
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );
}

#[test]
fn add_with_increment_failure_invalid_quantity() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = database.create_user().finish();
    let ticket_type = event.ticket_types(&connection).unwrap().remove(0);
    let update_parameters = TicketTypeEditableAttributes {
        increment: Some(4),
        ..Default::default()
    };
    let ticket_type = ticket_type.update(update_parameters, &connection).unwrap();
    let ticket_type_id = ticket_type.id;

    let input = Json(cart::AddToCartRequest {
        items: vec![cart::AddToCartRequestItem {
            ticket_type_id,
            quantity: 2,
        }],
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response: HttpResponse = cart::add((database.connection.into(), input, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let expected_json = json!({
        "error": "Validation error",
        "fields":{
            "quantity":[{"code":"quantity_invalid_increment","message":null,"params":{}}]
        }
    }).to_string();
    assert_eq!(body, expected_json);
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
        items: vec![cart::AddToCartRequestItem {
            ticket_type_id,
            quantity: 2,
        }],
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
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

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response = cart::remove((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Contains additional item quantity so cart response still includes cart object
    let body = support::unwrap_body_to_string(&response).unwrap();
    let cart_response: CartResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(cart.id, cart_response.cart_id);

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
fn remove_with_increment() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = event.ticket_types(&connection).unwrap().remove(0);
    let update_parameters = TicketTypeEditableAttributes {
        increment: Some(4),
        ..Default::default()
    };
    let ticket_type = ticket_type.update(update_parameters, &connection).unwrap();
    let ticket_type_id = ticket_type.id;
    let user = database.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();
    cart.add_tickets(ticket_type_id, 12, &connection).unwrap();

    let order_item = cart.items(&connection).unwrap().remove(0);
    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), &connection).unwrap();
    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), &connection).unwrap();
    assert_eq!(order_item.quantity, 12);
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 12
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );

    let input = Json(cart::RemoveCartRequest {
        cart_item_id: order_item.id,
        quantity: Some(4),
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response = cart::remove((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Contains additional item quantity so cart response still includes cart object
    let body = support::unwrap_body_to_string(&response).unwrap();
    let cart_response: CartResponse = serde_json::from_str(&body).unwrap();
    assert_eq!(cart.id, cart_response.cart_id);

    let order_item = cart.items(&connection).unwrap().remove(0);
    assert_eq!(order_item.quantity, 8);
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 8
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );
}

#[test]
fn remove_with_increment_failure_invalid_quantity() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = event.ticket_types(&connection).unwrap().remove(0);
    let update_parameters = TicketTypeEditableAttributes {
        increment: Some(4),
        ..Default::default()
    };
    let ticket_type = ticket_type.update(update_parameters, &connection).unwrap();
    let ticket_type_id = ticket_type.id;
    let user = database.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&connection)
        .unwrap();
    cart.add_tickets(ticket_type_id, 12, &connection).unwrap();

    let order_item = cart.items(&connection).unwrap().remove(0);
    let ticket_pricing =
        TicketPricing::find(order_item.ticket_pricing_id.unwrap(), &connection).unwrap();
    let fee_schedule_range =
        FeeScheduleRange::find(order_item.fee_schedule_range_id.unwrap(), &connection).unwrap();
    assert_eq!(order_item.quantity, 12);
    let fee_item = order_item.find_fee_item(&connection).unwrap().unwrap();
    assert_eq!(
        fee_item.unit_price_in_cents,
        fee_schedule_range.fee_in_cents * 12
    );
    assert_eq!(
        order_item.unit_price_in_cents,
        ticket_pricing.price_in_cents
    );

    let input = Json(cart::RemoveCartRequest {
        cart_item_id: order_item.id,
        quantity: Some(7),
    });

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response: HttpResponse =
        cart::remove((database.connection.into(), input, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    let expected_json = json!({
        "error": "Validation error",
        "fields":{
            "quantity":[{"code":"quantity_invalid_increment","message":null,"params":{}}]
        }
    }).to_string();
    assert_eq!(body, expected_json);
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

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response = cart::remove((database.connection.into(), input, auth_user)).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(cart.items(&connection).unwrap().is_empty());

    // Cart empty so was deleted
    let cart_result = Order::find_cart_for_user(user.id, &connection);
    assert_eq!(cart_result.err().unwrap().code, 2000);
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
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);

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

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
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

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
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

    let _order = database
        .create_cart()
        .for_user(&user)
        .for_event(&event)
        .finish();
    let request = TestRequest::create();

    let input = Json(cart::CheckoutCartRequest {
        amount: 100,
        method: PaymentRequest::External {
            reference: "TestRef".to_string(),
        },
    });

    // Must be admin to check out external
    let user = support::create_auth_user_from_user(&user, Roles::Admin, None, &database);

    let response = cart::checkout((
        database.connection.into(),
        input,
        user,
        request.extract_state(),
    )).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
