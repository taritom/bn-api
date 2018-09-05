use bigneon_db::models::{Artist, ArtistEditableAttributes};
use support::project::TestProject;
use uuid::Uuid;
use validator::Validate;

#[test]
fn commit() {
    let project = TestProject::new();
    let name = "Name";
    let bio = "Bio";
    let website_url = "http://www.example.com";

    let artist = Artist::create(name, bio, website_url)
        .commit(&project)
        .unwrap();
    assert_eq!(name, artist.name);
    assert_eq!(bio, artist.bio);
    assert_eq!(website_url, artist.website_url.unwrap());
    assert_eq!(artist.id.to_string().is_empty(), false);
}

#[test]
fn new_artist_validate() {
    let name = "Name";
    let bio = "Bio";
    let website_url = "invalid.com";

    let mut artist = Artist::create(name, bio, website_url);
    artist.youtube_video_urls = Some(vec!["h".into()]);

    let result = artist.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err().inner();

    assert!(errors.contains_key("website_url"));
    assert_eq!(errors["website_url"].len(), 1);
    assert_eq!(errors["website_url"][0].code, "url");

    assert!(errors.contains_key("youtube_video_urls"));
    assert_eq!(errors["youtube_video_urls"].len(), 1);
    assert_eq!(errors["youtube_video_urls"][0].code, "url");
}

#[test]
fn artist_editable_attributes_validate() {
    let mut artist_parameters = ArtistEditableAttributes::new();
    artist_parameters.name = Some("New Name".into());
    artist_parameters.bio = Some("Bio".into());
    artist_parameters.website_url = Some("invalid.com".into());
    artist_parameters.youtube_video_urls = Some(vec!["invalid".to_string()]);

    let result = artist_parameters.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err().inner();

    assert!(errors.contains_key("website_url"));
    assert_eq!(errors["website_url"].len(), 1);
    assert_eq!(errors["website_url"][0].code, "url");

    assert!(errors.contains_key("youtube_video_urls"));
    assert_eq!(errors["youtube_video_urls"].len(), 1);
    assert_eq!(errors["youtube_video_urls"][0].code, "url");
}

#[test]
fn find() {
    let project = TestProject::new();
    let artist = project.create_artist().finish();

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
    let artist = project.create_artist().with_name(name.into()).finish();
    assert_eq!(name, artist.name);
    assert_eq!(artist.id.to_string().is_empty(), false);

    let found_artists = Artist::all(&project).unwrap();
    assert_eq!(1, found_artists.len());
    assert_eq!(found_artists[0].id, artist.id);
    assert_eq!(found_artists[0].name, artist.name);

    let name2 = "Name 2";
    let artist2 = project.create_artist().with_name(name2.into()).finish();
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
    let artist = project.create_artist().with_name(name.into()).finish();

    println!(
        "Created at: {}, updated at:{}",
        artist.created_at, artist.updated_at
    );

    let mut artist_parameters = ArtistEditableAttributes::new();
    artist_parameters.name = Some("New Name".into());
    artist_parameters.bio = Some("Bio".into());
    artist_parameters.website_url = Some("http://www.example.com".into());
    let updated_artist = artist.update(&artist_parameters, &project).unwrap();

    assert_eq!(updated_artist.id, artist.id);
    assert_ne!(updated_artist.name, artist.name);
    assert_eq!(updated_artist.name, artist_parameters.name.unwrap());
}

#[test]
fn destroy() {
    let project = TestProject::new();
    let artist = project.create_artist().finish();
    assert!(artist.destroy(&project).unwrap() > 0);
    assert!(Artist::find(&artist.id, &project).is_err());
}
