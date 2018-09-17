use actix_web::FromRequest;
use actix_web::{http::StatusCode, Json, Path};
use bigneon_api::controllers::cart;
use bigneon_api::controllers::cart::PathParameters;
use bigneon_api::controllers::cart::PaymentRequest;
use bigneon_db::models::*;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn add() {
    let database = TestDatabase::new();
    let event = database
        .create_event()
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let user = database.create_user().finish();
    let request = TestRequest::create();

    let input = Json(cart::AddToCartRequest {
        ticket_type_id: event.ticket_types(&database.connection).unwrap()[0].id,
        quantity: 2,
    });

    let user = support::create_auth_user_from_user(&user, Roles::User, &database);
    let response = cart::add((database.connection.into(), input, user)).unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
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
