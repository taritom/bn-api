use bigneon_db::models::{Order, OrderTypes};
use support::project::TestProject;

#[test]
pub fn reserve_tickets() {
    let db = TestProject::new();

    let organization = db
        .create_organization()
        .with_fee_schedule(&db.create_fee_schedule().finish())
        .finish();
    let event = db
        .create_event()
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();
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
