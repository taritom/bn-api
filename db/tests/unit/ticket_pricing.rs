use bigneon_db::models::{TicketPricing, TicketType};
use chrono::NaiveDate;
use support::project::TestProject;

#[test]
fn create() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(project.get_connection()).unwrap()[0];
    let sd1 = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let ed1 = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let sd2 = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ed2 = NaiveDate::from_ymd(2016, 7, 10).and_hms(4, 10, 11);

    let ticket_pricing =
        TicketPricing::create(ticket_type.id, "Early Bird".to_string(), sd1, ed1, 100)
            .commit(project.get_connection())
            .unwrap();

    let pricing2 =
        TicketPricing::create(ticket_type.id, "Wormless Bird".to_string(), sd2, ed2, 500)
            .commit(project.get_connection())
            .unwrap();

    let pricing = ticket_type
        .ticket_pricing(project.get_connection())
        .unwrap();
    assert_eq!(pricing, vec![ticket_pricing, pricing2]);
}

#[test]
fn find() {
    let project = TestProject::new();
    let event = project.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(project.get_connection()).unwrap()[0];
    let sd1 = NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11);
    let ed1 = NaiveDate::from_ymd(2016, 7, 9).and_hms(4, 10, 11);
    let ticket_pricing =
        TicketPricing::create(ticket_type.id, "Early Bird".to_string(), sd1, ed1, 100)
            .commit(project.get_connection())
            .unwrap();
    let found_ticket_pricing =
        TicketPricing::find(ticket_pricing.id, project.get_connection()).unwrap();

    assert_eq!(found_ticket_pricing, ticket_pricing);
}

#[test]
fn get_current_ticket_pricing() {
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

    let ticket_types = TicketType::find_by_event_id(event.id, db.get_connection()).unwrap();

    let ticket_pricing =
        TicketPricing::get_current_ticket_pricing(ticket_types[0].id, db.get_connection()).unwrap();

    assert_eq!(ticket_pricing.name, "Standard".to_string())
}
