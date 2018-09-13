use bigneon_db::models::TicketPricing;
use support::project::TestProject;

#[test]
fn create() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(project.get_connection()).unwrap()[0];
    let ticket_pricing = TicketPricing::create(ticket_type.id, "Early Bird".to_string(), 100)
        .commit(project.get_connection())
        .unwrap();

    let pricing2 = TicketPricing::create(ticket_type.id, "Wormless Bird".to_string(), 500)
        .commit(project.get_connection())
        .unwrap();

    let pricing = ticket_type
        .ticket_pricing(project.get_connection())
        .unwrap();
    assert_eq!(pricing, vec![ticket_pricing, pricing2]);
}
