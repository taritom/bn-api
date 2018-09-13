use bigneon_db::models::TicketType;
use support::project::TestProject;
#[test]
fn create() {
    let db = TestProject::new();
    let event = db.create_event().finish();
    let ticket_type = event
        .add_ticket_type("VIP".to_string(), 100, &db.get_connection())
        .unwrap();

    assert_eq!(ticket_type.event_id, event.id);
    assert_eq!(ticket_type.name, "VIP".to_string())
}

#[test]
fn find() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(&db.get_connection()).unwrap()[0];

    let found_ticket_type = TicketType::find(ticket_type.id, &db.get_connection()).unwrap();
    assert_eq!(&found_ticket_type, ticket_type);
}
