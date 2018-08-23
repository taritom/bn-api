use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::organizations::{self, PathParameters, UpdateOwnerRequest};
use bigneon_db::models::{NewOrganization, Organization, OrganizationEditableAttributes, Roles};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn index(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_name("Organization 1".to_string())
        .with_owner(&user)
        .finish();
    let organization2 = database
        .create_organization()
        .with_name("Organization 2".to_string())
        .with_owner(&user)
        .finish();

    let expected_organizations = vec![organization, organization2];
    let organization_expected_json = serde_json::to_string(&expected_organizations).unwrap();

    let user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let response: HttpResponse = organizations::index((state, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, organization_expected_json);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

pub fn index_for_all_orgs(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();

    let user = database.create_user().finish();
    let user2 = database.create_user().finish();

    let organization = database
        .create_organization()
        .with_name("Organization 1".to_string())
        .with_owner(&user)
        .finish();
    let organization2 = database
        .create_organization()
        .with_name("Organization 2".to_string())
        .with_owner(&user2)
        .finish();

    let mut expected_organizations = vec![organization, organization2];
    if role == Roles::OrgMember {
        let index = expected_organizations
            .iter()
            .position(|x| x.owner_user_id == user2.id)
            .unwrap();
        expected_organizations.remove(index);
    }
    let organization_expected_json = serde_json::to_string(&expected_organizations).unwrap();

    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let response: HttpResponse = organizations::index_for_all_orgs((state, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, organization_expected_json);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

#[test]
fn show() {
    let database = TestDatabase::new();
    let organization = database.create_organization().finish();
    let organization_expected_json = serde_json::to_string(&organization).unwrap();

    let user = support::create_auth_user(Roles::OrgMember, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let response: HttpResponse = organizations::show((state, path, user)).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, organization_expected_json);
}

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let name = "Organization Example";
    let user = database.create_user().finish();

    let auth_user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(NewOrganization {
        owner_user_id: user.id,
        name: name.clone().to_string(),
        address: None,
        city: None,
        state: None,
        zip: None,
        country: None,
        phone: None,
    });

    let response: HttpResponse = organizations::create((state, json, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let org: Organization = serde_json::from_str(&body).unwrap();
        assert_eq!(org.name, name);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

pub fn update(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let organization = database.create_organization().finish();
    let new_name = "New Name";

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let json = Json(OrganizationEditableAttributes {
        name: Some(new_name.clone().to_string()),
        address: Some("address".to_string()),
        city: Some("city".to_string()),
        state: Some("state".to_string()),
        country: Some("country".to_string()),
        zip: Some("zip".to_string()),
        phone: Some("phone".to_string()),
    });

    let response: HttpResponse = organizations::update((state, path, json, user)).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_organization: Organization = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_organization.name, new_name);
}

pub fn remove_user(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let user3 = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_owner(&user)
        .with_user(&user2)
        .with_user(&user3)
        .finish();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(user3.id);
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse = organizations::remove_user((state, path, json, user)).into();
    let count = 1;
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let removed_entries: usize = serde_json::from_str(&body).unwrap();
        assert_eq!(removed_entries, count);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

pub fn update_owner(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let new_owner = database.create_user().finish();
    let organization = database.create_organization().with_owner(&user).finish();

    let auth_user = support::create_auth_user_from_user(&new_owner, role, &database);
    let test_request = TestRequest::create(database);
    let update_owner_request = UpdateOwnerRequest {
        owner_user_id: new_owner.id,
    };

    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let json = Json(update_owner_request);

    let response: HttpResponse = organizations::update_owner((state, path, json, auth_user)).into();

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_organization: Organization = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_organization.owner_user_id, new_owner.id);
}
