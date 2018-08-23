extern crate chrono;
use bigneon_db::models::{Event, EventHistory, Order, Venue};
use support::project::TestProject;
use unit::event_histories::chrono::prelude::*;

#[test]
fn create() {
    let project = TestProject::new();
    let venue = Venue::create("Name").commit(&project).unwrap();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&project)
        .unwrap();
    let order = Order::create(user.id, event.id).commit(&project).unwrap();
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
