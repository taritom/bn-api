use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path, Query};
use bigneon_api::controllers::users;
use bigneon_api::controllers::users::{CurrentUser, PathParameters, SearchUserByEmail};
use bigneon_db::models::{DisplayUser, Roles};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
fn current_user() {
    let database = TestDatabase::new();
    let db_user = database.create_user().finish();

    let user = support::create_auth_user_from_user(&db_user, Roles::Guest, &database);

    let response: HttpResponse = users::current_user((database.connection.into(), user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let current_user: CurrentUser = serde_json::from_str(&body).unwrap();
    let user = current_user.user;
    assert_eq!(user.id, db_user.id);
}

pub fn list_organizations(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().with_owner(&user).finish();

    let auth_user = support::create_auth_user_from_user(&user, role, &database);

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = user.id;

    let response: HttpResponse =
        users::list_organizations((database.connection.into(), path, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        #[derive(Serialize)]
        pub struct DisplayOrganizationLink {
            pub id: Uuid,
            pub name: String,
            pub role: String,
        }
        let role_owner_string = String::from("owner");
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
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let expected_json_string = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, expected_json_string);
    }
    assert_eq!(true, true);
}

pub fn show_from_email(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let email = "test@test.com";
    let db_user = database
        .create_user()
        .with_email(email.to_string())
        .finish();
    let user = support::create_auth_user_from_user(&db_user, role, &database);
    let test_request = TestRequest::create_with_uri(&format!("/?email={}", email));
    let data = Query::<SearchUserByEmail>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse =
        users::find_by_email((database.connection.into(), data, user)).into();
    let display_user: DisplayUser = db_user.into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        let user: DisplayUser = serde_json::from_str(&body).unwrap();
        assert_eq!(user, display_user);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let event_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, event_expected_json);
    }
}

pub fn show(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let display_user = database.create_user().finish().for_display();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = display_user.id;
    let response: HttpResponse = users::show((database.connection.into(), path, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        let user: DisplayUser = serde_json::from_str(&body).unwrap();
        assert_eq!(user, display_user);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let event_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, event_expected_json);
    }
}
