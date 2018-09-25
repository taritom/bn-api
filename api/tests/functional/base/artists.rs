use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::artists;
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);

    let name = "Artist Example";
    let bio = "Bio";
    let website_url = "http://www.example.com";

    let json = Json(NewArtist {
        organization_id: None,
        name: name.to_string(),
        bio: bio.to_string(),
        website_url: Some(website_url.to_string()),
        ..Default::default()
    });

    let response: HttpResponse =
        artists::create((database.connection.into(), json, auth_user)).into();

    if should_test_succeed {
        let body = support::unwrap_body_to_string(&response).unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let artist: Artist = serde_json::from_str(&body).unwrap();
        assert_eq!(artist.name, name);
        assert!(!artist.is_private);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn create_with_organization(role: Roles, should_test_succeed: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);

    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let name = "Artist Example";
    let bio = "Bio";
    let website_url = "http://www.example.com";

    let json = Json(NewArtist {
        organization_id: Some(organization.id),
        name: name.to_string(),
        bio: bio.to_string(),
        website_url: Some(website_url.to_string()),
        ..Default::default()
    });

    let response: HttpResponse =
        artists::create((database.connection.into(), json, auth_user.clone())).into();

    if should_test_succeed {
        let body = support::unwrap_body_to_string(&response).unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let artist: Artist = serde_json::from_str(&body).unwrap();
        assert_eq!(artist.name, name);
        assert!(artist.is_private);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn create_with_validation_errors(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();

    let name = "Artist Example";
    let bio = "Bio";
    let website_url = "invalid-format.com";
    let json = Json(NewArtist {
        name: name.to_string(),
        bio: bio.to_string(),
        website_url: Some(website_url.to_string()),
        youtube_video_urls: Some(vec!["invalid".to_string()]),
        ..Default::default()
    });

    let user = support::create_auth_user(role, &database);
    let response: HttpResponse = artists::create((database.connection.into(), json, user)).into();
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
        support::expects_unauthorized(&response);
    }
}

pub fn toggle_privacy(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let artist = database.create_artist().finish();

    let auth_user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let response: HttpResponse =
        artists::toggle_privacy((database.connection.into(), path, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_artist: Artist = serde_json::from_str(&body).unwrap();
        assert_ne!(updated_artist.is_private, artist.is_private)
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn update(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let artist = database.create_artist().finish();
    let name = "New Name";
    let bio = "New Bio";
    let website_url = "http://www.example2.com";

    let auth_user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let mut attributes: ArtistEditableAttributes = Default::default();
    attributes.name = Some(name.to_string());
    attributes.bio = Some(bio.to_string());
    attributes.website_url = Some(website_url.to_string());
    attributes.youtube_video_urls = Some(Vec::new());
    let json = Json(attributes);

    let response: HttpResponse =
        artists::update((database.connection.into(), path, json, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_artist: Artist = serde_json::from_str(&body).unwrap();
        assert_eq!(updated_artist.name, name);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn update_with_organization(
    role: Roles,
    should_test_succeed: bool,
    same_organization: bool,
    is_private: bool,
) {
    let database = TestDatabase::new();

    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);

    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let mut artist = database
        .create_artist()
        .with_organization(&organization)
        .finish();

    if is_private {
        artist = artist.set_privacy(true, &database.connection).unwrap();
    }

    let name = "New Name";
    let bio = "New Bio";
    let website_url = "http://www.example2.com";

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let mut attributes: ArtistEditableAttributes = Default::default();
    attributes.name = Some(name.to_string());
    attributes.bio = Some(bio.to_string());
    attributes.website_url = Some(website_url.to_string());
    attributes.youtube_video_urls = Some(Vec::new());
    let json = Json(attributes);

    let response: HttpResponse =
        artists::update((database.connection.into(), path, json, auth_user.clone())).into();
    if should_test_succeed {
        let body = support::unwrap_body_to_string(&response).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let updated_artist: Artist = serde_json::from_str(&body).unwrap();
        assert_eq!(updated_artist.name, name);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn update_with_validation_errors(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let artist = database.create_artist().finish();
    let name = "New Name";
    let bio = "New Bio";
    let website_url = "invalid-format.com";

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let mut attributes: ArtistEditableAttributes = Default::default();
    attributes.name = Some(name.to_string());
    attributes.bio = Some(bio.to_string());
    attributes.website_url = Some(website_url.to_string());
    attributes.youtube_video_urls = Some(vec!["invalid".to_string()]);
    let json = Json(attributes);

    let response: HttpResponse =
        artists::update((database.connection.into(), path, json, user)).into();
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
        support::expects_unauthorized(&response);
    }
}
