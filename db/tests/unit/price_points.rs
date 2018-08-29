use bigneon_db::models::PricePoint;
use support::project::TestProject;

#[test]
fn create() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    let ticket_type = &event.ticket_types(&db).unwrap()[0];
    let price_point = PricePoint::create(ticket_type.id, "Early Bird".to_string(), 100)
        .commit(&db)
        .unwrap();

    let price_point2 = PricePoint::create(ticket_type.id, "Wormless Bird".to_string(), 500)
        .commit(&db)
        .unwrap();

    let price_points = ticket_type.price_points(&db).unwrap();
    assert_eq!(price_points, vec![price_point, price_point2]);
}
