use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path};
use bigneon_api::controllers::artists::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{Artist, Roles};
use functional::base;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn index() {
    let database = TestDatabase::new();
    let artist = Artist::create("Artist", "Bio", "http://www.example.com")
        .commit(&*database.get_connection())
        .unwrap();
    let artist2 = Artist::create("Artist 2", "Bio", "http://www.example.com")
        .commit(&*database.get_connection())
        .unwrap();

    let expected_artists = vec![artist, artist2];
    let artist_expected_json = serde_json::to_string(&expected_artists).unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let response: HttpResponse = artists::index(state).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, artist_expected_json);
}

#[test]
fn show() {
    let database = TestDatabase::new();
    let artist = Artist::create("Name", "Bio", "http://www.example.com")
        .commit(&*database.get_connection())
        .unwrap();
    let artist_expected_json = serde_json::to_string(&artist).unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = artist.id;

    let response: HttpResponse = artists::show((state, path)).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, artist_expected_json);
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
