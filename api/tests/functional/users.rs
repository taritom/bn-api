use actix_web::{http::StatusCode, HttpResponse, Json};
use bigneon_api::controllers::users::{self, CurrentUser};
use bigneon_api::models::{RegisterRequest, UserProfileAttributes};
use bigneon_db::models::Roles;
use functional::base;
use serde_json;
use support;
use support::database::TestDatabase;

#[cfg(test)]
mod user_search_by_email_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        base::users::show_from_email(Roles::OrgMember, false);
    }
    #[test]
    fn index_guest() {
        base::users::show_from_email(Roles::Guest, false);
    }
    #[test]
    fn index_admin() {
        base::users::show_from_email(Roles::Admin, true);
    }
    #[test]
    fn index_user() {
        base::users::show_from_email(Roles::User, false);
    }
    #[test]
    fn index_org_owner() {
        base::users::show_from_email(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod users_show_tests {
    use super::*;
    #[test]
    fn show_org_member() {
        base::users::show(Roles::OrgMember, false);
    }
    #[test]
    fn show_guest() {
        base::users::show(Roles::Guest, false);
    }
    #[test]
    fn show_admin() {
        base::users::show(Roles::Admin, true);
    }
    #[test]
    fn show_user() {
        base::users::show(Roles::User, false);
    }
    #[test]
    fn show_org_owner() {
        base::users::show(Roles::OrgOwner, true);
    }
}

#[test]
fn register_address_exists() {
    let database = TestDatabase::new();

    let existing_user = database.create_user().finish();

    let json = Json(RegisterRequest::new(
        &"First",
        &"Last",
        &existing_user.email.unwrap(),
        &"555",
        &"not_important",
    ));

    let response: HttpResponse = users::register((database.connection.into(), json)).into();

    if response.status() == StatusCode::OK {
        panic!("Duplicate email was allowed when it should not be")
    }
}

#[test]
fn register_succeeds() {
    let database = TestDatabase::new();

    let json = Json(RegisterRequest::new(
        &"First",
        &"Last",
        &"fake@localhost",
        &"555",
        &"not_important",
    ));

    let response: HttpResponse = users::register((database.connection.into(), json)).into();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[test]
fn register_with_validation_errors() {
    let database = TestDatabase::new();

    let json = Json(RegisterRequest::new(
        &"First",
        &"Last",
        &"bad-email",
        &"555",
        &"not_important",
    ));

    let response: HttpResponse = users::register((database.connection.into(), json)).into();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let expected_json = json!({
        "error": "Validation error",
        "fields":{
            "email":[{"code":"email","message":null,"params":{"value":"bad-email"}}]
        }
    }).to_string();
    assert_eq!(body, expected_json);
}

#[test]
pub fn update_current_user() {
    let database = TestDatabase::new();
    let user = support::create_auth_user(Roles::Guest, &database);
    let email = "new-email@tari.com";
    let mut attributes: UserProfileAttributes = Default::default();
    attributes.email = Some(email.clone().into());
    let json = Json(attributes);

    let response: HttpResponse =
        users::update_current_user((database.connection.into(), json, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated_user: CurrentUser = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_user.user.email, Some(email.into()));
}

#[test]
pub fn update_current_user_with_validation_errors() {
    let database = TestDatabase::new();
    let user = support::create_auth_user(Roles::Guest, &database);
    let mut attributes: UserProfileAttributes = Default::default();
    attributes.email = Some("bad-email".into());
    let json = Json(attributes);

    let response: HttpResponse =
        users::update_current_user((database.connection.into(), json, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let expected_json = json!({
        "error": "Validation error",
        "fields":{
            "email":[{"code":"email","message":null,"params":{"value":"bad-email"}}]
        }
    }).to_string();
    assert_eq!(body, expected_json);
}

#[test]
fn update_current_user_address_exists() {
    let database = TestDatabase::new();
    let existing_user = database.create_user().finish();

    let user = support::create_auth_user(Roles::Guest, &database);
    let mut attributes: UserProfileAttributes = Default::default();
    attributes.email = existing_user.email;
    let json = Json(attributes);

    let response: HttpResponse =
        users::update_current_user((database.connection.into(), json, user)).into();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
