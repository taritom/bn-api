use actix_web::{FromRequest, Json, Path};
use bigneon_api::controllers::artists::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{Artist, NewArtist};
use serde_json;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn index() {
    let database = TestDatabase::new();
    let artist = Artist::create(&"Artist")
        .commit(&*database.get_connection())
        .unwrap();
    let artist2 = Artist::create(&"Artist 2")
        .commit(&*database.get_connection())
        .unwrap();

    let expected_artists = vec![artist, artist2];
    let artist_expected_json = serde_json::to_string(&expected_artists).unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let response = artists::index(state);

    match response {
        Ok(body) => {
            assert_eq!(body, artist_expected_json);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn show() {
    let database = TestDatabase::new();
    let artist = Artist::create(&"Name")
        .commit(&*database.get_connection())
        .unwrap();
    let artist_expected_json = serde_json::to_string(&artist).unwrap();

    let test_request = TestRequest::create_with_route(
        database,
        &"/artists/{id}",
        &format!("/artists/{}", artist.id.to_string()),
    );
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();

    let response = artists::show((state, path));

    match response {
        Ok(body) => {
            assert_eq!(body, artist_expected_json);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn create() {
    let database = TestDatabase::new();

    let name = "Artist Example";
    let json = Json(NewArtist {
        name: name.clone().to_string(),
    });

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let response = artists::create((state, json));

    match response {
        Ok(body) => {
            let artist: Artist = serde_json::from_str(&body).unwrap();

            assert_eq!(artist.name, name);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn update() {
    let database = TestDatabase::new();
    let artist = Artist::create(&"Name")
        .commit(&*database.get_connection())
        .unwrap();
    let new_name = "New Name";

    let test_request = TestRequest::create_with_route(
        database,
        &"/artists/{id}",
        &format!("/artists/{}", artist.id.to_string()),
    );
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    let json = Json(NewArtist {
        name: new_name.clone().to_string(),
    });

    let response = artists::update((state, path, json));

    match response {
        Ok(body) => {
            let updated_artist: Artist = serde_json::from_str(&body).unwrap();
            assert_eq!(updated_artist.name, new_name);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn destroy() {
    let database = TestDatabase::new();
    let connection = &*database.get_connection();
    let artist = Artist::create(&"Name").commit(connection).unwrap();

    let test_request = TestRequest::create_with_route(
        database,
        &"/artists/{id}",
        &format!("/artists/{}", artist.id.to_string()),
    );
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();

    let response = artists::destroy((state, path));
    let expected_json = "{}";

    match response {
        Ok(body) => {
            assert_eq!(body, expected_json);
        }
        _ => panic!("Unexpected response body"),
    }

    let artist = Artist::find(&artist.id, connection);
    match artist {
        Ok(_a) => panic!("Not found error did not occur as expected"),
        Err(_e) => (),
    }
}
