use bigneon_db::models::TicketType;
use chrono::NaiveDate;
use support::project::TestProject;

#[test]
fn create() {
    let db = TestProject::new();
    let event = db.create_event().finish();
    let sd = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let ed = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ticket_type = event
        .add_ticket_type("VIP".to_string(), 100, sd, ed, &db.get_connection())
        .unwrap();

    assert_eq!(ticket_type.event_id, event.id);
    assert_eq!(ticket_type.name, "VIP".to_string())
}

#[test]
pub fn remaining_ticket_count() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project.create_event().with_ticket_pricing().finish();
    let ticket_type = event.ticket_types(connection).unwrap().remove(0);
    let order = project.create_order().for_event(&event).finish();
    assert_eq!(90, ticket_type.remaining_ticket_count(connection).unwrap());

    order.add_tickets(ticket_type.id, 10, connection).unwrap();
    assert_eq!(80, ticket_type.remaining_ticket_count(connection).unwrap());

    let order_item = order.items(connection).unwrap().remove(0);
    assert!(
        order
            .remove_tickets(order_item, Some(4), connection)
            .is_ok()
    );
    assert_eq!(84, ticket_type.remaining_ticket_count(connection).unwrap());
}

#[test]
fn find() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(&db.get_connection()).unwrap()[0];

    let found_ticket_type = TicketType::find(ticket_type.id, &db.get_connection()).unwrap();
    assert_eq!(&found_ticket_type, ticket_type);
}
