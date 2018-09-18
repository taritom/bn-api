use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path, Query};
use bigneon_api::controllers::users;
use bigneon_api::controllers::users::{PathParameters, SearchUserByEmail};
use bigneon_db::models::{DisplayUser, Roles};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

pub fn list_organizations(role: Roles, should_test_true: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);

    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.with_user(&user2)
    .finish();

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = user2.id;

    let response: HttpResponse =
        users::list_organizations((database.connection.into(), path, auth_user.clone())).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        #[derive(Serialize)]
        pub struct DisplayOrganizationLink {
            pub id: Uuid,
            pub name: String,
            pub role: String,
        }
        let role_owner_string = String::from("member");
        let expected_data = DisplayOrganizationLink {
            id: organization.id,
            name: organization.name,
            role: role_owner_string,
        };
        let expected_json_string = format!(
            "[{}]",
            serde_json::to_string(&expected_data).unwrap().to_string()
        );
        assert_eq!(body, expected_json_string);
    } else {
        support::expects_unauthorized(&response);
    }
    assert_eq!(true, true);
}

pub fn find_by_email(role: Roles, should_test_true: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let email = "test@test.com";

    let user = database.create_user().finish();
    let user2 = database
        .create_user()
        .with_email(email.to_string())
        .finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);

    let _organization = if same_organization {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.with_user(&user2)
    .finish();

    let test_request = TestRequest::create_with_uri(&format!("/?email={}", email));
    let data = Query::<SearchUserByEmail>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        users::find_by_email((database.connection.into(), data, auth_user.clone())).into();
    let display_user: DisplayUser = user2.into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        let user: DisplayUser = serde_json::from_str(&body).unwrap();
        assert_eq!(user, display_user);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn show(role: Roles, should_test_true: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();

    let _organization = if same_organization {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.with_user(&user2)
    .finish();

    let display_user = user2.for_display();

    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = display_user.id;
    let response: HttpResponse =
        users::show((database.connection.into(), path, auth_user.clone())).into();
    if should_test_true {
        let body = support::unwrap_body_to_string(&response).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let user: DisplayUser = serde_json::from_str(&body).unwrap();
        assert_eq!(user, display_user);
    } else {
        support::expects_unauthorized(&response);
    }
}
