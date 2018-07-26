use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::auth::user::User as AuthUser;
use bigneon_api::controllers::organizations::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{NewOrganization, Organization, Roles, User};
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
    let mut organization = Organization::create(user.id, &"Organization")
        .commit(&*database.get_connection())
        .unwrap();
    organization.address = Some(<String>::from("Test Address"));
    organization.city = Some(<String>::from("Test Address"));
    organization.state = Some(<String>::from("Test state"));
    organization.country = Some(<String>::from("Test country"));
    organization.zip = Some(<String>::from("0124"));
    organization.phone = Some(<String>::from("+27123456789"));
    organization = Organization::update(&organization, &*database.get_connection()).unwrap();
    let mut organization2 = Organization::create(user.id, &"Organization 2")
        .commit(&*database.get_connection())
        .unwrap();
    organization.address = Some(<String>::from("Test Address"));
    organization2.city = Some(<String>::from("Test Address"));
    organization2.state = Some(<String>::from("Test state"));
    organization2.country = Some(<String>::from("Test country"));
    organization2.zip = Some(<String>::from("0124"));
    organization2.phone = Some(<String>::from("+27123456789"));
    organization2 = Organization::update(&organization2, &*database.get_connection()).unwrap();

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
    let mut organization = Organization::create(user.id, &"testOrganization")
        .commit(&*database.get_connection())
        .unwrap();
    organization.address = Some(<String>::from("Test Address"));
    organization.city = Some(<String>::from("Test Address"));
    organization.state = Some(<String>::from("Test state"));
    organization.country = Some(<String>::from("Test country"));
    organization.zip = Some(<String>::from("0124"));
    organization.phone = Some(<String>::from("+27123456789"));
    organization = Organization::update(&organization, &*database.get_connection()).unwrap();
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
#[test]
fn update() {
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
    let json = Json(Organization {
        id: organization.id,
        owner_user_id: user.id,
        name: new_name.clone().to_string(),
        address: organization.address.clone(),
        city: organization.city.clone(),
        state: organization.state.clone(),
        country: organization.country.clone(),
        zip: organization.zip.clone(),
        phone: organization.phone.clone(),
    });

    let user = AuthUser::new(Uuid::new_v4(), vec![Roles::OrgOwner]);
    let response = organizations::update((state, path, json, user));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_organization: Organization = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_organization.name, new_name);
}
