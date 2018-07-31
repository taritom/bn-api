use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::auth::user::User as AuthUser;
use bigneon_api::controllers::organizations::UpdateOwnerRequest;
use bigneon_api::controllers::organizations::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{
    NewOrganization, Organization, OrganizationEditableAttributes, OrganizationUser, Roles, User,
};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

pub fn index(role: Roles) {
    let database = TestDatabase::new();
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, &"Organization")
        .commit(&*database.get_connection())
        .unwrap();

    let mut attrs = OrganizationEditableAttributes::new();
    attrs.address = Some(<String>::from("Test Address"));
    attrs.city = Some(<String>::from("Test Address"));
    attrs.state = Some(<String>::from("Test state"));
    attrs.country = Some(<String>::from("Test country"));
    attrs.zip = Some(<String>::from("0124"));
    attrs.phone = Some(<String>::from("+27123456789"));
    let organization = organization
        .update(attrs, &*database.get_connection())
        .unwrap();
    let organization2 = Organization::create(user.id, &"Organization 2")
        .commit(&*database.get_connection())
        .unwrap();

    let mut attrs = OrganizationEditableAttributes::new();
    attrs.address = Some(<String>::from("Test Address"));
    attrs.city = Some(<String>::from("Test Address"));
    attrs.state = Some(<String>::from("Test state"));
    attrs.country = Some(<String>::from("Test country"));
    attrs.zip = Some(<String>::from("0124"));
    attrs.phone = Some(<String>::from("+27123456789"));
    let organization2 = organization2
        .update(attrs, &*database.get_connection())
        .unwrap();

    let expected_organizations = vec![organization, organization2];
    let organization_expected_json = serde_json::to_string(&expected_organizations).unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let user = AuthUser::new(user.id, vec![role]);
    let should_test_true = user.is_in_role(Roles::OrgMember);
    let response = organizations::index((state, user));
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
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
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, &"testOrganization")
        .commit(&*database.get_connection())
        .unwrap();

    let mut attrs = OrganizationEditableAttributes::new();

    attrs.address = Some(<String>::from("Test Address"));
    attrs.city = Some(<String>::from("Test Address"));
    attrs.state = Some(<String>::from("Test state"));
    attrs.country = Some(<String>::from("Test country"));
    attrs.zip = Some(<String>::from("0124"));
    attrs.phone = Some(<String>::from("+27123456789"));
    let organization = organization
        .update(attrs, &*database.get_connection())
        .unwrap();
    let organization_expected_json = serde_json::to_string(&organization).unwrap();

    let test_request = TestRequest::create_with_route(
        database,
        &"/organizations/{id}",
        &format!("/organizations/{}", organization.id.to_string()),
    );
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    let user = AuthUser::new(Uuid::new_v4(), vec![Roles::OrgMember]);
    let response = organizations::show((state, path, user));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, organization_expected_json);
}

pub fn create(role: Roles) {
    let database = TestDatabase::new();
    let name = "Organization Example";
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(NewOrganization {
        owner_user_id: user.id,
        name: name.clone().to_string(),
    });

    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let should_test_true = user.is_in_role(Roles::Admin);
    let response = organizations::create((state, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
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
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, &"Name")
        .commit(&*database.get_connection())
        .unwrap();
    let new_name = "New Name";

    let test_request = TestRequest::create_with_route(
        database,
        &"/organizations/{id}",
        &format!("/organizations/{}", organization.id.to_string()),
    );
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    let json = Json(OrganizationEditableAttributes {
        name: Some(new_name.clone().to_string()),
        address: Some("address".to_string()),
        city: Some("city".to_string()),
        state: Some("state".to_string()),
        country: Some("country".to_string()),
        zip: Some("zip".to_string()),
        phone: Some("phone".to_string()),
    });

    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let response = organizations::update((state, path, json, user));

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_organization: Organization = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_organization.name, new_name);
}

pub fn remove_user(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let user2 = User::create("Jeff2", "jeff2@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let user3 = User::create("Jeff3", "jeff3@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, &"OrgName")
        .commit(&*database.get_connection())
        .unwrap();
    //create links from org to users
    let _orguser = OrganizationUser::create(organization.id, user2.id)
        .commit(&*database.get_connection())
        .unwrap();
    let _orguser2 = OrganizationUser::create(organization.id, user3.id)
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create_with_route(
        database,
        &"/organizations/{id}",
        &format!("/organizations/{}", organization.id.to_string()),
    );
    let state = test_request.extract_state();
    let json = Json(user3.id);

    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    let response = organizations::remove_user((state, path, json, user));

    let count = 1;
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
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
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let new_owner = User::create(
        "New Jeff",
        "jeff2@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, &"Name")
        .commit(&*database.get_connection())
        .unwrap();

    let test_request = TestRequest::create_with_route(
        database,
        &"/organizations/{id}/owner",
        &format!("/organizations/{}/owner", organization.id.to_string()),
    );
    let update_owner_request = UpdateOwnerRequest {
        owner_user_id: new_owner.id,
    };

    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();

    let json = Json(update_owner_request);

    let response =
        organizations::update_owner((state, path, json, AuthUser::new(new_owner.id, vec![role])));

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_organization: Organization = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_organization.owner_user_id, new_owner.id);
}
