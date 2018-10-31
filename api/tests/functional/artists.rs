use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path, Query};
use bigneon_api::controllers::artists;
use bigneon_api::models::PathParameters;
use bigneon_db::prelude::*;
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
    let test_request = TestRequest::create_with_uri(&format!("/limits?"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        artists::index((database.connection.into(), query_parameters, None)).into();

    let wrapped_expected_artists = Payload {
        data: expected_artists,
        paging: Paging {
            page: 0,
            limit: 2,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 2,
            tags: Vec::new(),
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_artists).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
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
        .with_organization(&org1)
        .finish();

    let artist4 = database
        .create_artist()
        .make_private()
        .with_name("Artist4".to_string())
        .with_organization(&org1)
        .finish();

    let test_request = TestRequest::create_with_uri(&format!("/limits?"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    //first try with no user
    let response: HttpResponse =
        artists::index((database.connection.clone().into(), query_parameters, None)).into();

    let mut expected_artists = vec![artist, artist2, artist3];

    let body = support::unwrap_body_to_string(&response).unwrap();
    let wrapped_expected_artists = Payload {
        data: expected_artists.clone(),
        paging: Paging {
            page: 0,
            limit: 3,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 3,
            tags: Vec::new(),
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_artists).unwrap();
    assert_eq!(body, expected_json);

    //now try with user that does not belong to org
    let user = support::create_auth_user(Roles::User, None, &database);
    let user_id = user.id();
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse = artists::index((
        database.connection.clone().into(),
        query_parameters,
        Some(user.clone()),
    )).into();
;

    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);

    //now with user that DOES belong to org
    let _ = org1.add_user(user_id, &database.connection.clone());
    expected_artists.push(artist4);
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        artists::index((database.connection.into(), query_parameters, Some(user))).into();
    let wrapped_expected_artists = Payload {
        data: expected_artists,
        paging: Paging {
            page: 0,
            limit: 4,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 4,
            tags: Vec::new(),
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_artists).unwrap();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}

#[test]
pub fn show() {
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
        .with_organization(&organization)
        .finish();
    let artist2 = database
        .create_artist()
        .with_name("Artist 2".to_string())
        .with_organization(&organization)
        .make_private()
        .finish();

    let user2 = database.create_user().finish();

    let all_artists = vec![artist, artist2];
    let wrapped_expected_artists = Payload {
        data: all_artists,
        paging: Paging {
            page: 0,
            limit: 2,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 2,
            tags: Vec::new(),
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_artists).unwrap();
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let user = support::create_auth_user_from_user(
        &user2,
        Roles::OrgOwner,
        Some(&organization),
        &database,
    );

    let test_request = TestRequest::create_with_uri(&format!("/limits?"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse = artists::show_from_organizations((
        database.connection.into(),
        path,
        query_parameters,
        Some(user),
    )).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(expected_json, body);
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::artists::create(Roles::OrgMember, false);
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
    #[test]
    fn create_with_organization_org_member() {
        base::artists::create_with_organization(Roles::OrgMember, true);
    }
    #[test]
    fn create_with_organization_admin() {
        base::artists::create_with_organization(Roles::Admin, true);
    }
    #[test]
    fn create_with_organization_user() {
        base::artists::create_with_organization(Roles::User, false);
    }
    #[test]
    fn create_with_organization_org_owner() {
        base::artists::create_with_organization(Roles::OrgOwner, true);
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
mod toggle_privacy_tests {
    use super::*;
    #[test]
    fn toggle_privacy_org_member() {
        base::artists::toggle_privacy(Roles::OrgMember, false);
    }
    #[test]
    fn toggle_privacy_admin() {
        base::artists::toggle_privacy(Roles::Admin, true);
    }
    #[test]
    fn toggle_privacy_user() {
        base::artists::toggle_privacy(Roles::User, false);
    }
    #[test]
    fn toggle_privacy_org_owner() {
        base::artists::toggle_privacy(Roles::OrgOwner, false);
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
    #[test]
    fn update_with_organization_org_member() {
        base::artists::update_with_organization(Roles::OrgMember, true, true);
    }
    #[test]
    fn update_with_organization_admin() {
        base::artists::update_with_organization(Roles::Admin, true, true);
    }
    #[test]
    fn update_with_organization_user() {
        base::artists::update_with_organization(Roles::User, false, true);
    }
    #[test]
    fn update_with_organization_org_owner() {
        base::artists::update_with_organization(Roles::OrgOwner, true, true);
    }
    #[test]
    fn update_public_artist_with_organization_org_member() {
        base::artists::update_with_organization(Roles::OrgMember, false, false);
    }
    #[test]
    fn update_public_artist_with_organization_admin() {
        base::artists::update_with_organization(Roles::Admin, true, false);
    }
    #[test]
    fn update_public_artist_with_organization_user() {
        base::artists::update_with_organization(Roles::User, false, false);
    }
    #[test]
    fn update_public_artist_with_organization_org_owner() {
        base::artists::update_with_organization(Roles::OrgOwner, false, false);
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
