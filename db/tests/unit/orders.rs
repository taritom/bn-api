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
        .commit(project.get_connection())
        .unwrap();
    assert_eq!(order.user_id, user.id);
    assert_eq!(order.id.to_string().is_empty(), false);
}

#[test]
fn add_to_cart() {
    let project = TestProject::new();
    let event = project
        .create_event()
        .with_tickets()
        .with_price_points()
        .finish();
    let user = project.create_user().finish();
    let cart = Order::create(user.id, OrderTypes::Cart)
        .commit(project.get_connection())
        .unwrap();
    let ticket = &event.ticket_types(project.get_connection()).unwrap()[0];
    cart.add_tickets(ticket.id, 10, project.get_connection())
        .unwrap();

    let db_cart = Order::find_cart_for_user(user.id, project.get_connection()).unwrap();
    assert_eq!(cart.id, db_cart.id);
    assert_eq!(
        cart.items(project.get_connection()).unwrap(),
        db_cart.items(project.get_connection()).unwrap()
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
fn checkout() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let event = project
        .create_event()
        .with_tickets()
        .with_price_points()
        .finish();
    let mut cart = Order::create(user.id, OrderTypes::Cart)
        .commit(project.get_connection())
        .unwrap();
    let ticket = &event.ticket_types(project.get_connection()).unwrap()[0];
    cart.add_tickets(ticket.id, 10, project.get_connection())
        .unwrap();

    cart.checkout(project.get_connection()).unwrap();
    assert_eq!(cart.user_id, user.id);
    assert_eq!(cart.status(), OrderStatus::PendingPayment);
}
