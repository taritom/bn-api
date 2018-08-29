use bigneon_db::models::TicketType;
use support::project::TestProject;
#[test]
fn create() {
    let db = TestProject::new();
    let event = db.create_event().finish();
    let ticket_type = TicketType::create(event.id, "VIP".to_string())
        .commit(&db)
        .unwrap();

    assert_eq!(ticket_type.event_id, event.id);
    assert_eq!(ticket_type.name, "VIP".to_string())
}
