use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::artists::{self, PathParameters};
use bigneon_db::models::{Artist, ArtistEditableAttributes, Roles};
use functional::base;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn index() {
    let database = TestDatabase::new();
    let artist = database
        .create_artist()
        .with_name("Artist1".to_string())
        .finish();
    let artist2 = database
        .create_artist()
        .with_name("Artist2".to_string())
        .finish();

    let expected_artists = vec![artist, artist2];
    let artist_expected_json = serde_json::to_string(&expected_artists).unwrap();
    let response: HttpResponse = artists::index((database.connection.into(), None)).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, artist_expected_json);
}

#[test]
fn index_with_org_linked_and_private_venues() {
    let database = TestDatabase::new();
    let artist = database
        .create_artist()
        .with_name("Artist1".to_string())
        .finish();
    let artist2 = database
        .create_artist()
        .with_name("Artist2".to_string())
        .finish();

    let org1 = database.create_organization().finish();
    let artist3 = database
        .create_artist()
        .with_name("Artist3".to_string())
        .with_organization(org1.id)
        .finish();

    let artist4 = database
        .create_artist()
        .make_private()
        .with_name("Artist4".to_string())
        .with_organization(org1.id)
        .finish();

    //first try with no user
    let response: HttpResponse = artists::index((database.connection.clone().into(), None)).into();

    let mut expected_artists = vec![artist, artist2, artist3];
    let artist_expected_json = serde_json::to_string(&expected_artists).unwrap();

    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, artist_expected_json);

    //now try with user that does not belong to org
    let user = support::create_auth_user(Roles::OrgOwner, &database);
    let user_id = user.id();
    let response: HttpResponse =
        artists::index((database.connection.clone().into(), Some(user.clone()))).into();

    let artist_expected_json = serde_json::to_string(&expected_artists).unwrap();

    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, artist_expected_json);

    //now with user that DOES belong to org
    let _ = org1.add_user(user_id, &database.connection.clone());
    expected_artists.push(artist4);
    let response: HttpResponse = artists::index((database.connection.into(), Some(user))).into();
    let artist_expected_json = serde_json::to_string(&expected_artists).unwrap();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, artist_expected_json);
}

#[test]
fn show() {
    let database = TestDatabase::new();
    let artist = database.create_artist().finish();
    let artist_expected_json = serde_json::to_string(&artist).unwrap();

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let response: HttpResponse = artists::show((database.connection.into(), path)).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, artist_expected_json);
}

#[test]
pub fn show_from_organizations_private_artist_same_org() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().with_user(&user).finish();
    let artist = database
        .create_artist()
        .with_name("Artist 1".to_string())
        .with_organization(organization.id)
        .finish();
    let artist2 = database
        .create_artist()
        .with_name("Artist 2".to_string())
        .with_organization(organization.id)
        .make_private()
        .finish();

    let user2 = database.create_user().finish();
    let _ = organization.add_user(user2.id, &database.connection);

    let all_artists = vec![artist, artist2];
    let artist_expected_json = serde_json::to_string(&all_artists).unwrap();

    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let user = support::create_auth_user_from_user(&user2, Roles::OrgOwner, &database);

    let response: HttpResponse =
        artists::show_from_organizations((database.connection.into(), path, Some(user))).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(artist_expected_json, body);
}

#[test]
fn update_as_org_owner_of_artists_org() {
    //This test is for the special case that an OrgOwner CAN update a venue that belongs to his organization
    let database = TestDatabase::new();
    let organization = database.create_organization().finish();
    let artist = database
        .create_artist()
        .with_organization(organization.id)
        .finish();
    let new_name = "New Name";

    let user2 = database.create_user().finish();
    let _ = user2.add_role(Roles::OrgOwner, &database.connection);
    let _ = organization.add_user(user2.id, &database.connection);

    let user = support::create_auth_user_from_user(&user2, Roles::OrgOwner, &database);

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let mut attributes: ArtistEditableAttributes = Default::default();
    attributes.name = Some(new_name.to_string());
    let json = Json(attributes);

    let response: HttpResponse =
        artists::update((database.connection.into(), path, json, user)).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_artist: Artist = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_artist.name, new_name);
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::artists::create(Roles::OrgMember, false);
    }
    #[test]
    fn create_guest() {
        base::artists::create(Roles::Guest, false);
    }
    #[test]
    fn create_admin() {
        base::artists::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        base::artists::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        base::artists::create(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod create_with_validation_errors_tests {
    use super::*;
    #[test]
    fn create_with_validation_errors_org_member() {
        base::artists::create_with_validation_errors(Roles::OrgMember, false);
    }
    #[test]
    fn create_with_validation_errors_guest() {
        base::artists::create_with_validation_errors(Roles::Guest, false);
    }
    #[test]
    fn create_with_validation_errors_admin() {
        base::artists::create_with_validation_errors(Roles::Admin, true);
    }
    #[test]
    fn create_with_validation_errors_user() {
        base::artists::create_with_validation_errors(Roles::User, false);
    }
    #[test]
    fn create_with_validation_errors_org_owner() {
        base::artists::create_with_validation_errors(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        base::artists::update(Roles::OrgMember, false);
    }
    #[test]
    fn update_guest() {
        base::artists::update(Roles::Guest, false);
    }
    #[test]
    fn update_admin() {
        base::artists::update(Roles::Admin, true);
    }
    #[test]
    fn update_user() {
        base::artists::update(Roles::User, false);
    }
    #[test]
    fn update_org_owner() {
        base::artists::update(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod update_with_validation_errors_tests {
    use super::*;
    #[test]
    fn update_with_validation_errors_org_member() {
        base::artists::update_with_validation_errors(Roles::OrgMember, false);
    }
    #[test]
    fn update_with_validation_errors_guest() {
        base::artists::update_with_validation_errors(Roles::Guest, false);
    }
    #[test]
    fn update_with_validation_errors_admin() {
        base::artists::update_with_validation_errors(Roles::Admin, true);
    }
    #[test]
    fn update_with_validation_errors_user() {
        base::artists::update_with_validation_errors(Roles::User, false);
    }
    #[test]
    fn update_with_validation_errors_org_owner() {
        base::artists::update_with_validation_errors(Roles::OrgOwner, false);
    }
}
