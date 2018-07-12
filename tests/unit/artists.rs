use actix_web::Body::Binary;
use actix_web::{http, test, FromRequest, HttpRequest, Json, Path, State};
use bigneon_api::controllers::artists::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_api::server::AppState;
use bigneon_db::models::{Artist, NewArtist};
use serde_json;
use std::str;
use support::database::TestDatabase;

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

    let response = test::TestRequest::with_state(AppState {
        database: Box::new(database),
    }).run(artists::index)
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    match response.body() {
        Binary(binary) => {
            let body = str::from_utf8(binary.as_ref()).unwrap();
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

    let artist_show = move |request: HttpRequest<AppState>| {
        let state = State::<AppState>::extract(&request);
        let mut path = Path::<PathParameters>::extract(&request).unwrap();
        path.id = artist.id;
        artists::show((state, path))
    };

    let response = test::TestRequest::with_state(AppState {
        database: Box::new(database),
    }).param(&"id", &"1f418fc2-9a51-4f2e-9ac0-683bd8aa876d")
        .run(artist_show)
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    match response.body() {
        Binary(binary) => {
            let body = str::from_utf8(binary.as_ref()).unwrap();
            assert_eq!(body, artist_expected_json);
        }
        _ => panic!("Unexpected response body"),
    }
}

#[test]
fn create() {
    let database = TestDatabase::new();
    let name = "Artist Example";

    let artist_create = move |request: HttpRequest<AppState>| {
        let state = State::<AppState>::extract(&request);
        let json = Json(NewArtist {
            name: name.clone().to_string(),
        });
        artists::create((state, json))
    };

    let response = test::TestRequest::with_state(AppState {
        database: Box::new(database),
    }).run(artist_create)
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    match response.body() {
        Binary(binary) => {
            let body = str::from_utf8(binary.as_ref()).unwrap();
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

    let artist_update = move |request: HttpRequest<AppState>| {
        let state = State::<AppState>::extract(&request);
        let mut path = Path::<PathParameters>::extract(&request).unwrap();
        path.id = artist.id;
        let json = Json(NewArtist {
            name: new_name.clone().to_string(),
        });
        artists::update((state, path, json))
    };

    let response = test::TestRequest::with_state(AppState {
        database: Box::new(database),
    }).param(&"id", &"1f418fc2-9a51-4f2e-9ac0-683bd8aa876d")
        .run(artist_update)
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    match response.body() {
        Binary(binary) => {
            let body = str::from_utf8(binary.as_ref()).unwrap();
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
    let artist_id = artist.id.clone();

    let artist_destroy = move |request: HttpRequest<AppState>| {
        let state = State::<AppState>::extract(&request);
        let mut path = Path::<PathParameters>::extract(&request).unwrap();
        path.id = artist.id;
        artists::destroy((state, path))
    };

    let response = test::TestRequest::with_state(AppState {
        database: Box::new(database),
    }).param(&"id", &"1f418fc2-9a51-4f2e-9ac0-683bd8aa876d")
        .run(artist_destroy)
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let expected_json = "{}";
    match response.body() {
        Binary(binary) => {
            let body = str::from_utf8(binary.as_ref()).unwrap();
            assert_eq!(body, expected_json);
        }
        _ => panic!("Unexpected response body"),
    }

    let artist = Artist::find(&artist_id, connection);
    match artist {
        Ok(_a) => panic!("Not found error did not occur as expected"),
        Err(_e) => (),
    }
}
