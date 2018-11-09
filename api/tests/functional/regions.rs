use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path, Query};
use bigneon_api::controllers::regions;
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use functional::base;
use serde_json;
use std::collections::HashMap;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn index() {
    let database = TestDatabase::new();
    let region = database
        .create_region()
        .with_name("Region1".into())
        .finish();
    let region2 = database
        .create_region()
        .with_name("Region2".into())
        .finish();

    let expected_regions = vec![region, region2];
    let test_request = TestRequest::create_with_uri(&format!("/limits?"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        regions::index((database.connection.into(), query_parameters)).into();
    let wrapped_expected_regions = Payload {
        data: expected_regions,
        paging: Paging {
            page: 0,
            limit: 2,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: 2,
            tags: HashMap::new(),
        },
    };
    let expected_json = serde_json::to_string(&wrapped_expected_regions).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, expected_json);
}

#[test]
fn show() {
    let database = TestDatabase::new();
    let region = database.create_region().finish();
    let region_expected_json = serde_json::to_string(&region).unwrap();

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = region.id;

    let response: HttpResponse = regions::show((database.connection.into(), path)).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, region_expected_json);
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::regions::create(Roles::OrgMember, false);
    }
    #[test]
    fn create_admin() {
        base::regions::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        base::regions::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        base::regions::create(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        base::regions::update(Roles::OrgMember, false);
    }
    #[test]
    fn update_admin() {
        base::regions::update(Roles::Admin, true);
    }
    #[test]
    fn update_user() {
        base::regions::update(Roles::User, false);
    }
    #[test]
    fn update_org_owner() {
        base::regions::update(Roles::OrgOwner, false);
    }
}
