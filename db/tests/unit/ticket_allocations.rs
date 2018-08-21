use bigneon_db::models::*;
use support::project::TestProject;

#[test]
pub fn create_test() {
    let mut project = TestProject::new();
    let event = project.create_event().finish();
    let allocation = TicketAllocation::create(event.id, 100)
        .commit(&project)
        .unwrap();
    assert_eq!(allocation.tari_asset_id(), None);
    assert_eq!(allocation.ticket_delta(), 100);
}

#[test]
pub fn update_test() {
    let mut project = TestProject::new();
    let event = project.create_event().finish();
    let mut allocation = TicketAllocation::create(event.id, 100)
        .commit(&project)
        .unwrap();
    allocation.set_asset_id("asset1".into());
    let res = allocation.update(&project).unwrap();
    assert_eq!(res.tari_asset_id(), Some("asset1".into()));
}
