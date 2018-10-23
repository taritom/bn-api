use bigneon_db::dev::TestProject;
use bigneon_db::models::*;

#[test]
pub fn create() {
    let db = TestProject::new();
    let event = db.create_event().finish();
    Hold::create(
        "test".to_string(),
        event.id,
        "IHAVEACODE".to_string(),
        0,
        None,
        Some(4),
    ).commit(db.get_connection())
    .unwrap();
}

#[test]
pub fn update() {
    let db = TestProject::new();
    let event = db.create_event().finish();
    let hold = Hold::create(
        "test".to_string(),
        event.id,
        "IHAVEACODE".to_string(),
        0,
        None,
        Some(4),
    ).commit(db.get_connection())
    .unwrap();

    let update_patch = UpdateHoldAttributes {
        discount_in_cents: Some(10),
        max_per_order: Some(None),
        end_at: Some(None),
        name: Some("New name".to_string()),
    };
    let new_hold = hold.update(update_patch, db.get_connection()).unwrap();
    assert_eq!(new_hold.name, "New name".to_string());
    assert_eq!(new_hold.max_per_order, None);
    assert_eq!(new_hold.end_at, None);
    assert_eq!(new_hold.discount_in_cents, 10);
}

#[test]
pub fn set_quantity() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    let hold = Hold::create(
        "test".to_string(),
        event.id,
        "IHAVEACODE".to_string(),
        0,
        None,
        Some(4),
    ).commit(db.get_connection())
    .unwrap();
    let ticket_type_id = event.ticket_types(db.get_connection()).unwrap()[0].id;

    hold.set_quantity(ticket_type_id, 30, db.get_connection())
        .unwrap();

    assert_eq!(
        hold.quantity(ticket_type_id, db.get_connection()).unwrap(),
        30
    );
}
