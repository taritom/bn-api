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

    let artist = Artist::create(name, None, bio, website_url)
        .commit(project.get_connection())
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

    let mut artist = Artist::create(name, None, bio, website_url);

    artist.image_url = Some("invalid".into());
    artist.thumb_image_url = Some("invalid".into());

    artist.youtube_video_urls = Some(vec!["h".into()]);

    let result = artist.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err().field_errors();

    assert!(errors.contains_key("image_url"));
    assert_eq!(errors["image_url"].len(), 1);
    assert_eq!(errors["image_url"][0].code, "url");

    assert!(errors.contains_key("youtube_video_urls"));
    assert_eq!(errors["youtube_video_urls"].len(), 1);
    assert_eq!(errors["youtube_video_urls"][0].code, "url");

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
    let errors = result.unwrap_err().field_errors();

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
    let connection = project.get_connection();
    let artist = project.create_artist().finish();

    let found_artist = Artist::find(&artist.id, connection).expect("Artist was not found");
    assert_eq!(found_artist.id, artist.id);
    assert_eq!(found_artist.name, artist.name);

    assert!(
        match Artist::find(&Uuid::new_v4(), connection) {
            Ok(_artist) => false,
            Err(_e) => true,
        },
        "Artist incorrectly returned when id invalid"
    );
}

#[test]
fn set_privacy() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let mut artist = project.create_artist().finish();
    assert!(!artist.is_private);

    artist = artist.set_privacy(true, connection).unwrap();
    assert!(artist.is_private);

    artist = artist.set_privacy(false, connection).unwrap();
    assert!(!artist.is_private);
}

#[test]
fn all() {
    let project = TestProject::new();
    let name = "Name";
    let owner = project.create_user().finish();
    let user = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_owner(&owner)
        .with_user(&user)
        .finish();

    let artist = project.create_artist().with_name(name.into()).finish();
    assert_eq!(name, artist.name);
    assert_eq!(artist.id.to_string().is_empty(), false);

    let found_artists = Artist::all(None, project.get_connection()).unwrap();
    assert_eq!(1, found_artists.len());
    assert_eq!(found_artists[0].id, artist.id);
    assert_eq!(found_artists[0].name, artist.name);

    let name2 = "Name 2";
    let artist2 = project
        .create_artist()
        .with_name(name2.into())
        .with_organization(&organization)
        .make_private()
        .finish();
    assert_eq!(name2, artist2.name);
    assert_eq!(artist2.id.to_string().is_empty(), false);

    let found_artists = Artist::all(None, project.get_connection()).unwrap();
    assert_eq!(1, found_artists.len());
    assert_eq!(found_artists[0].id, artist.id);
    assert_eq!(found_artists[0].name, artist.name);

    let found_artists_owner = Artist::all(Some(owner.id), project.get_connection()).unwrap();
    let found_artists_user = Artist::all(Some(user.id), project.get_connection()).unwrap();
    assert_eq!(found_artists_user, found_artists_owner);
    assert_eq!(2, found_artists_user.len());
    assert_eq!(found_artists_user[0].id, artist.id);
    assert_eq!(found_artists_user[0].name, artist.name);
    assert_eq!(found_artists_user[1].id, artist2.id);
    assert_eq!(found_artists_user[1].name, artist2.name);
}

#[test]
fn update() {
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
    let updated_artist = artist
        .update(&artist_parameters, &project.get_connection())
        .unwrap();

    assert_eq!(updated_artist.id, artist.id);
    assert_ne!(updated_artist.name, artist.name);
    assert_eq!(updated_artist.name, artist_parameters.name.unwrap());
}

#[test]
fn destroy() {
    let project = TestProject::new();
    let artist = project.create_artist().finish();
    assert!(artist.destroy(project.get_connection()).unwrap() > 0);
    assert!(Artist::find(&artist.id, project.get_connection()).is_err());
}

#[test]
fn organization() {
    let project = TestProject::new();
    let organization = project.create_organization().finish();
    let artist = project
        .create_artist()
        .with_organization(&organization)
        .finish();
    let artist2 = project.create_artist().finish();

    assert_eq!(
        Ok(Some(organization)),
        artist.organization(project.get_connection())
    );
    assert_eq!(Ok(None), artist2.organization(project.get_connection()));
}

#[test]
fn find_for_organization() {
    let project = TestProject::new();
    let owner = project.create_user().finish();
    let member = project.create_user().finish();
    let user = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_owner(&owner)
        .with_user(&member)
        .finish();
    let artist1 = project
        .create_artist()
        .with_name("Artist1".to_string())
        .with_organization(&organization)
        .finish();

    let artist2 = project
        .create_artist()
        .with_name("Artist2".to_string())
        .with_organization(&organization)
        .finish();

    let artist3 = project
        .create_artist()
        .with_name("Artist3".to_string())
        .with_organization(&organization)
        .make_private()
        .finish();

    // Add another artist for another org to make sure it isn't included
    let organization2 = project.create_organization().with_owner(&user).finish();
    let artist4 = project
        .create_artist()
        .with_name("Artist4".to_string())
        .with_organization(&organization2)
        .finish();

    let user = project.create_user().finish();

    let mut all_artists = vec![artist1, artist2];

    let found_artists =
        Artist::find_for_organization(None, organization.id, project.get_connection()).unwrap();
    assert_eq!(found_artists, all_artists);

    let found_artists =
        Artist::find_for_organization(None, organization.id, project.get_connection()).unwrap();
    assert_eq!(found_artists, all_artists);
    assert!(!found_artists.contains(&artist3));
    assert!(!found_artists.contains(&artist4));

    // Private artist is not shown for users
    let found_artists =
        Artist::find_for_organization(Some(user.id), organization.id, project.get_connection())
            .unwrap();
    assert_eq!(found_artists, all_artists);

    // Private artist is shown for owners and members
    all_artists.push(artist3);
    let found_artists =
        Artist::find_for_organization(Some(owner.id), organization.id, project.get_connection())
            .unwrap();
    assert_eq!(found_artists, all_artists);

    let found_artists =
        Artist::find_for_organization(Some(member.id), organization.id, project.get_connection())
            .unwrap();
    assert_eq!(found_artists, all_artists);
}
