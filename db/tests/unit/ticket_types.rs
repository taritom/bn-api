use bigneon_db::models::TicketType;
use support::project::TestProject;
#[test]
fn create() {
    let project = TestProject::new();
    let event = project.create_event().finish();
    let ticket_type = TicketType::create(event.id, "VIP".to_string())
        .commit(project.get_connection())
        .unwrap();

    assert_eq!(ticket_type.event_id, event.id);
    assert_eq!(ticket_type.name, "VIP".to_string())
}
