extern crate chrono;
use bigneon_db::models::EventInterest;

use support::project::TestProject;

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let event = project.create_event().finish();

    let event_interest = EventInterest::create(event.id, user.id)
        .commit(&project)
        .unwrap();

    assert_eq!(event_interest.user_id, user.id);
    assert_eq!(event_interest.event_id, event.id);
}

#[test]
fn total_interest() {
    let project = TestProject::new();
    let user1 = project.create_user().finish();
    let user2 = project.create_user().finish();
    let event = project.create_event().finish();

    let _event_interest1 = EventInterest::create(event.id, user1.id)
        .commit(&project)
        .unwrap();

    let _event_interest2 = EventInterest::create(event.id, user2.id)
        .commit(&project)
        .unwrap();

    assert_eq!(
        EventInterest::total_interest(event.id, &project).unwrap(),
        2
    );
}

#[test]
fn user_interest() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let event = project.create_event().finish();

    let _event_interest1 = EventInterest::create(event.id, user.id)
        .commit(&project)
        .unwrap();

    assert_eq!(
        EventInterest::user_interest(event.id, user.id, &project).unwrap(),
        true
    );
}
