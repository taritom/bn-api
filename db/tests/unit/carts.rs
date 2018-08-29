use bigneon_db::models::{Cart, CartStatus, OrderStatus};
use support::project::TestProject;

#[test]
fn add_to_cart() {
    let db = TestProject::new();
    let event = db
        .create_event()
        .with_tickets()
        .with_price_points()
        .finish();
    let user = db.create_user().finish();
    let cart = Cart::create(user.id).commit(&db).unwrap();
    let tickets = &event.ticket_types(&db).unwrap()[0];
    let price_point = &tickets.price_points(&db).unwrap()[0];
    cart.add_item(price_point.id, 10, &db).unwrap();

    let db_cart = Cart::find_for_user(user.id, &db).unwrap();
    assert_eq!(cart.id, db_cart.id);
    assert_eq!(cart.items(&db).unwrap(), db_cart.items(&db).unwrap());
}

#[test]
fn find_by_user_when_cart_does_not_exist() {
    let db = TestProject::new();
    let user = db.create_user().finish();
    let cart_result = Cart::find_for_user(user.id, &db);
    assert_eq!(cart_result.err().unwrap().code, 2000);
}

#[test]
fn checkout() {
    let db = TestProject::new();
    let user = db.create_user().finish();
    let event = db
        .create_event()
        .with_tickets()
        .with_price_points()
        .finish();
    let mut cart = Cart::create(user.id).commit(&db).unwrap();
    let tickets = &event.ticket_types(&db).unwrap()[0];
    let price_point = &tickets.price_points(&db).unwrap()[0];
    cart.add_item(price_point.id, 10, &db).unwrap();

    let order = cart.checkout_and_create_order(&db).unwrap();
    assert_eq!(order.user_id, user.id);
    assert_eq!(order.status(), OrderStatus::Unpaid);
    assert_eq!(cart.status(), CartStatus::Completed);
    assert_eq!(cart.order_id, Some(order.id));
}
