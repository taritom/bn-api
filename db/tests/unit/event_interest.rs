extern crate chrono;
use bigneon_db::models::{Event, EventInterest, Venue};
use chrono::NaiveDate;
use support::project::TestProject;

#[test]
fn create() {
    let project = TestProject::new();
    let venue = Venue::create("Venue").commit(&project).unwrap();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&project)
        .unwrap();

    let event_interest = EventInterest::create(event.id, user.id)
        .commit(&project)
        .unwrap();

    assert_eq!(event_interest.user_id, user.id);
    assert_eq!(event_interest.event_id, event.id);
}
