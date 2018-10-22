use bigneon_db::dev::TestProject;
use bigneon_db::models::EventArtist;

#[test]
fn create() {
    let project = TestProject::new();
    let artist = project.create_artist().finish();
    let event = project.create_event().finish();
    let rank = 1;

    let event_artist = EventArtist::create(event.id, artist.id, rank, None)
        .commit(project.get_connection())
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

#[test]
fn find_all_by_event() {
    let project = TestProject::new();
    let artist1 = project.create_artist().finish();
    let artist2 = project.create_artist().finish();
    let event = project.create_event().finish();

    let event_artist1 = EventArtist::create(event.id, artist1.id, 1, None)
        .commit(project.get_connection())
        .unwrap();
    let event_artist2 = EventArtist::create(event.id, artist2.id, 2, None)
        .commit(project.get_connection())
        .unwrap();

    let result = EventArtist::find_all_from_event(event.id, project.get_connection()).unwrap();

    assert_eq!(vec![event_artist1, event_artist2], result);
}
