use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use chrono::prelude::*;

#[test]
fn find_for_settlement_by_event() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project.create_organization().finish();
    let event = project.create_event().finish();
    let settlement = Settlement::create(
        organization.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11),
        NaiveDate::from_ymd(2020, 7, 8).and_hms(4, 10, 11),
        SettlementStatus::PendingSettlement,
        Some("test comment".to_string()),
        true,
    )
    .commit(connection)
    .unwrap();
    let settlement_entry = project
        .create_settlement_entry()
        .with_face_value_in_cents(100)
        .with_event(&event)
        .with_settlement(&settlement)
        .finish();
    let mut results =
        SettlementEntry::find_for_settlement_by_event(&settlement, connection).unwrap();
    assert_eq!(results.len(), 1);
    let grouped_settlement_entry = results.pop().unwrap();
    assert_eq!(
        grouped_settlement_entry.event,
        event.for_display(connection).unwrap()
    );
    assert_eq!(grouped_settlement_entry.entries.len(), 1);
    assert_eq!(grouped_settlement_entry.entries[0].id, settlement_entry.id);
}

#[test]
fn create() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let venue = project.create_venue().finish();
    let user = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_member(&user, Roles::OrgOwner)
        .finish();
    let event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let settlement = Settlement::create(
        organization.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(4, 10, 11),
        NaiveDate::from_ymd(2020, 7, 8).and_hms(4, 10, 11),
        SettlementStatus::PendingSettlement,
        Some("test comment".to_string()),
        true,
    )
    .commit(connection)
    .unwrap();

    let settlement_entry = project
        .create_settlement_entry()
        .with_face_value_in_cents(100)
        .with_event(&event)
        .with_settlement(&settlement)
        .finish();

    assert_eq!(settlement_entry.event_id, event.id);
    assert_eq!(settlement_entry.ticket_type_id, None);
    assert_eq!(settlement_entry.face_value_in_cents, 100);
}
