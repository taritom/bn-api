extern crate chrono;
use bigneon_db::models::{Event, Order, Venue};
use support::project::TestProject;
use unit::orders::chrono::prelude::*;

#[test]
fn create() {
    let project = TestProject::new();
    let venue = Venue::create("Name").commit(&project).unwrap();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = project
        .create_event()
        .with_name("NewEvent".into())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let order = Order::create(user.id).commit(&project).unwrap();
    assert_eq!(order.user_id, user.id);
    assert_eq!(order.id.to_string().is_empty(), false);
}
