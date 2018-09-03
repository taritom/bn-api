use bigneon_db::models::Order;
use bigneon_db::models::{OrderStatus, OrderTypes};
use support::project::TestProject;

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
        .commit(&project)
        .unwrap();
    assert_eq!(order.user_id, user.id);
    assert_eq!(order.id.to_string().is_empty(), false);
}

#[test]
fn add_to_cart() {
    let db = TestProject::new();
    let event = db
        .create_event()
        .with_tickets()
        .with_price_points()
        .finish();
    let user = db.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&db)
        .unwrap();
    let ticket = &event.ticket_types(&db).unwrap()[0];
    cart.add_tickets(ticket.id, 10, &db).unwrap();

    let db_cart = Order::find_cart_for_user(user.id, &db).unwrap();
    assert_eq!(cart.id, db_cart.id);
    assert_eq!(cart.items(&db).unwrap(), db_cart.items(&db).unwrap());
}

#[test]
fn find_by_user_when_cart_does_not_exist() {
    let db = TestProject::new();
    let user = db.create_user().finish();
    let cart_result = Order::find_cart_for_user(user.id, &db);
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
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(&db)
        .unwrap();
    let ticket = &event.ticket_types(&db).unwrap()[0];
    cart.add_tickets(ticket.id, 10, &db).unwrap();

    cart.checkout(&db).unwrap();
    assert_eq!(cart.user_id, user.id);
    assert_eq!(cart.status(), OrderStatus::PendingPayment);
}
