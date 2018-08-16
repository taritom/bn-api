use bigneon_db::models::artists::UserEditableAttributes;
use bigneon_db::models::Artist;
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn commit() {
    let project = TestProject::new();
    let name = "Name";
    let artist = Artist::create(&name).commit(&project).unwrap();
    assert_eq!(name, artist.name);
    assert_eq!(artist.id.to_string().is_empty(), false);
}

#[test]
fn find() {
    let project = TestProject::new();
    let artist = Artist::create("Name").commit(&project).unwrap();

    let found_artist = Artist::find(&artist.id, &project).expect("Artist was not found");
    assert_eq!(found_artist.id, artist.id);
    assert_eq!(found_artist.name, artist.name);

    assert!(
        match Artist::find(&Uuid::new_v4(), &project) {
            Ok(_artist) => false,
            Err(_e) => true,
        },
        "Artist incorrectly returned when id invalid"
    );
}

#[test]
fn all() {
    let project = TestProject::new();
    let name = "Name";
    let artist = Artist::create(&name).commit(&project).unwrap();
    assert_eq!(name, artist.name);
    assert_eq!(artist.id.to_string().is_empty(), false);

    let found_artists = Artist::all(&project).unwrap();
    assert_eq!(1, found_artists.len());
    assert_eq!(found_artists[0].id, artist.id);
    assert_eq!(found_artists[0].name, artist.name);

    let name2 = "Name 2";
    let artist2 = Artist::create(&name2).commit(&project).unwrap();
    assert_eq!(name2, artist2.name);
    assert_eq!(artist2.id.to_string().is_empty(), false);

    let found_artists = Artist::all(&project).unwrap();
    assert_eq!(2, found_artists.len());
    assert_eq!(found_artists[0].id, artist.id);
    assert_eq!(found_artists[0].name, artist.name);
    assert_eq!(found_artists[1].id, artist2.id);
    assert_eq!(found_artists[1].name, artist2.name);
}

#[test]
fn update_attributes() {
    let project = TestProject::new();
    let name = "Old Name";
    let artist = Artist::create(&name).commit(&project).unwrap();

    let artist_parameters = UserEditableAttributes {
        name: "New Name".to_string(),
    };
    let updated_artist = artist.update(&artist_parameters, &project).unwrap();

    assert_eq!(updated_artist.id, artist.id);
    assert_ne!(updated_artist.name, artist.name);
    assert_eq!(updated_artist.name, artist_parameters.name);
}

#[test]
fn destroy() {
    let project = TestProject::new();
    let name = "Old Name";
    let artist = Artist::create(&name).commit(&project).unwrap();
    assert!(artist.destroy(&project).unwrap() > 0);
    assert!(Artist::find(&artist.id, &project).is_err());
}
