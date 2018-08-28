extern crate chrono;
use bigneon_db::models::{EventHistory, Order, Venue};
use support::project::TestProject;

#[test]
fn create() {
    let project = TestProject::new();
    let venue = Venue::create("Name").commit(&project).unwrap();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = project.create_event().finish();

    let order = Order::create(user.id).commit(&project).unwrap();
    let protocol_reference_hash = "HASH";
    let event_history = EventHistory::create(event.id, order.id, user.id, protocol_reference_hash)
        .commit(&project)
        .unwrap();
    assert_eq!(event_history.event_id, event.id);
    assert_eq!(event_history.order_id, order.id);
    assert_eq!(event_history.user_id, user.id);
    assert_eq!(
        event_history.protocol_reference_hash,
        protocol_reference_hash
    );
    assert_eq!(event_history.id.to_string().is_empty(), false);
}
