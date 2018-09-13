use bigneon_db::models::{Order, OrderTypes};
use support::project::TestProject;

#[test]
pub fn reserve_tickets() {
    let db = TestProject::new();

    let event = db.create_event().with_ticket_pricing().finish();
    let user = db.create_user().finish();
    let order = Order::create(user.id, OrderTypes::Cart)
        .commit(db.get_connection())
        .unwrap();
    let ticket_type_id = event.ticket_types(db.get_connection()).unwrap()[0].id;
    let tickets = order
        .add_tickets(ticket_type_id, 10, db.get_connection())
        .unwrap();
    assert_eq!(tickets.len(), 10);
}
