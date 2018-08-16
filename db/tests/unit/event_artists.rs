extern crate chrono;
use bigneon_db::models::{Artist, Event, EventArtist, Venue};
use support::project::TestProject;
use unit::event_artists::chrono::prelude::*;

#[test]
fn create() {
    let project = TestProject::new();
    let artist = Artist::create("Name").commit(&project).unwrap();
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
    let rank = 1;

    let event_artist = EventArtist::create(event.id, artist.id, rank)
        .commit(&project)
        .unwrap();

    assert_eq!(
        event_artist.event_id, event.id,
        "Event foreign key does not match"
    );
    assert_eq!(
        event_artist.artist_id, artist.id,
        "Artist foreign key does not match"
    );
    assert_eq!(event_artist.rank, rank, "Artist rank is not correct");
    assert_eq!(event_artist.id.to_string().is_empty(), false);
}
