use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::artists::{self, PathParameters};
use bigneon_db::models::{Artist, ArtistEditableAttributes, NewArtist, Roles};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();

    let name = "Artist Example";
    let bio = "Bio";
    let website_url = "http://www.example.com";

    let json = Json(NewArtist {
        name: name.clone().to_string(),
        bio: bio.clone().to_string(),
        website_url: Some(website_url.clone().to_string()),
        youtube_video_urls: Vec::new(),
        facebook_username: None,
        instagram_username: None,
        snapshat_username: None,
        soundcloud_username: None,
        bandcamp_username: None,
    });

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let response: HttpResponse = artists::create((state, json, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let artist: Artist = serde_json::from_str(&body).unwrap();
        assert_eq!(artist.name, name);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, expected_json);
    }
}

pub fn create_with_validation_errors(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();

    let name = "Artist Example";
    let bio = "Bio";
    let website_url = "invalid-format.com";
    let json = Json(NewArtist {
        name: name.clone().to_string(),
        bio: bio.clone().to_string(),
        website_url: Some(website_url.to_string()),
        youtube_video_urls: vec!["invalid".to_string()],
        facebook_username: None,
        instagram_username: None,
        snapshat_username: None,
        soundcloud_username: None,
        bandcamp_username: None,
    });

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let response: HttpResponse = artists::create((state, json, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let expected_json = json!({
            "error": "Validation error",
            "fields":{
                "website_url":[{"code":"url","message":null,"params":{"value":"invalid-format.com"}}],
                "youtube_video_urls":[{"code":"url","message":null,"params":{"value":["invalid"]}}]
            }
        }).to_string();
        assert_eq!(body, expected_json);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, expected_json);
    }
}

pub fn update(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let artist = database.create_artist().finish();
    let name = "New Name";
    let bio = "New Bio";
    let website_url = "http://www.example2.com";

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let mut attributes: ArtistEditableAttributes = Default::default();
    attributes.name = Some(name.clone().to_string());
    attributes.bio = Some(bio.clone().to_string());
    attributes.website_url = Some(website_url.clone().to_string());
    attributes.youtube_video_urls = Some(Vec::new());
    let json = Json(attributes);

    let response: HttpResponse = artists::update((state, path, json, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_artist: Artist = serde_json::from_str(&body).unwrap();
        assert_eq!(updated_artist.name, name);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, expected_json);
    }
}

pub fn update_with_validation_errors(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let artist = database.create_artist().finish();
    let name = "New Name";
    let bio = "New Bio";
    let website_url = "invalid-format.com";

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let mut attributes: ArtistEditableAttributes = Default::default();
    attributes.name = Some(name.clone().to_string());
    attributes.bio = Some(bio.clone().to_string());
    attributes.website_url = Some(website_url.clone().to_string());
    attributes.youtube_video_urls = Some(vec!["invalid".to_string()]);
    let json = Json(attributes);

    let response: HttpResponse = artists::update((state, path, json, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let expected_json = json!({
            "error": "Validation error",
            "fields":{
                "website_url":[{"code":"url","message":null,"params":{"value":"invalid-format.com"}}],
                "youtube_video_urls":[{"code":"url","message":null,"params":{"value":["invalid"]}}]
            }
        }).to_string();
        assert_eq!(body, expected_json);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, expected_json);
    }
}
