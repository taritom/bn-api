use actix_web::{http::StatusCode, HttpResponse, Json};
use bigneon_api::controllers::users::{self, CurrentUser};
use bigneon_api::models::{RegisterRequest, UserProfileAttributes};
use bigneon_db::models::{Roles, User};
use functional::base;
use serde_json;
use std::collections::HashMap;
use support;
use support::database::TestDatabase;

#[cfg(test)]
mod user_list_organizations_tests {
    use super::*;
    #[test]
    fn list_organizations_org_member() {
        base::users::list_organizations(Roles::OrgMember, false, true);
    }
    #[test]
    fn list_organizations_admin() {
        base::users::list_organizations(Roles::Admin, true, true);
    }
    #[test]
    fn list_organizations_user() {
        base::users::list_organizations(Roles::User, false, true);
    }
    #[test]
    fn list_organizations_org_owner() {
        base::users::list_organizations(Roles::OrgOwner, true, true);
    }
    #[test]
    fn list_organizations_other_organization_org_member() {
        base::users::list_organizations(Roles::OrgMember, false, false);
    }
    #[test]
    fn list_organizations_other_organization_admin() {
        base::users::list_organizations(Roles::Admin, true, false);
    }
    #[test]
    fn list_organizations_other_organization_user() {
        base::users::list_organizations(Roles::User, false, false);
    }
    #[test]
    fn list_organizations_other_organization_org_owner() {
        base::users::list_organizations(Roles::OrgOwner, false, false);
    }
}

#[cfg(test)]
mod find_by_email_tests {
    use super::*;
    #[test]
    fn find_by_email_org_member() {
        base::users::find_by_email(Roles::OrgMember, false, true);
    }
    #[test]
    fn find_by_email_admin() {
        base::users::find_by_email(Roles::Admin, true, true);
    }
    #[test]
    fn find_by_email_user() {
        base::users::find_by_email(Roles::User, false, true);
    }
    #[test]
    fn find_by_email_org_owner() {
        base::users::find_by_email(Roles::OrgOwner, true, true);
    }
    #[test]
    fn find_by_email_other_organization_org_member() {
        base::users::find_by_email(Roles::OrgMember, false, false);
    }
    #[test]
    fn find_by_email_other_organization_admin() {
        base::users::find_by_email(Roles::Admin, true, false);
    }
    #[test]
    fn find_by_email_other_organization_user() {
        base::users::find_by_email(Roles::User, false, false);
    }
    #[test]
    fn find_by_email_other_organization_org_owner() {
        base::users::find_by_email(Roles::OrgOwner, false, false);
    }
}

#[cfg(test)]
mod users_show_tests {
    use super::*;
    #[test]
    fn show_org_member() {
        base::users::show(Roles::OrgMember, false, true);
    }
    #[test]
    fn show_admin() {
        base::users::show(Roles::Admin, true, true);
    }
    #[test]
    fn show_user() {
        base::users::show(Roles::User, false, true);
    }
    #[test]
    fn show_org_owner() {
        base::users::show(Roles::OrgOwner, true, true);
    }
    #[test]
    fn show_other_organization_org_member() {
        base::users::show(Roles::OrgMember, false, false);
    }
    #[test]
    fn show_other_organization_admin() {
        base::users::show(Roles::Admin, true, false);
    }
    #[test]
    fn show_other_organization_user() {
        base::users::show(Roles::User, false, false);
    }
    #[test]
    fn show_other_organization_org_owner() {
        base::users::show(Roles::OrgOwner, false, false);
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
fn current_user() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, &database);

    let response: HttpResponse =
        users::current_user((database.connection.into(), auth_user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let current_user: CurrentUser = serde_json::from_str(&body).unwrap();
    let user = current_user.user;
    assert_eq!(user.id, user.id);
    assert_eq!(vec!["event:interest", "order:read"], current_user.scopes);
    assert!(current_user.organization_scopes.is_empty());
}

#[test]
fn current_user_admin() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::Admin, &database);

    let response: HttpResponse =
        users::current_user((database.connection.into(), auth_user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let current_user: CurrentUser = serde_json::from_str(&body).unwrap();
    let user = current_user.user;
    assert_eq!(user.id, user.id);
    assert_eq!(
        vec![
            "artist:write",
            "event:interest",
            "event:view-guests",
            "event:write",
            "order::make-external-payment",
            "order:read",
            "org:admin",
            "org:read",
            "org:write",
            "region:write",
            "ticket:admin",
            "user:read",
            "venue:write"
        ],
        current_user.scopes
    );
    assert!(current_user.organization_scopes.is_empty());
}

#[test]
fn current_user_organization_owner() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization_with_user(&user, true).finish();
    let user = User::find(user.id, &database.connection).unwrap();
    let auth_user = support::create_auth_user_from_user(&user, Roles::OrgOwner, &database);

    let response: HttpResponse =
        users::current_user((database.connection.clone().into(), auth_user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let current_user: CurrentUser = serde_json::from_str(&body).unwrap();
    let user = current_user.user;
    assert_eq!(user.id, user.id);
    assert_eq!(vec!["event:interest", "order:read"], current_user.scopes);
    let mut expected_results = HashMap::new();
    expected_results.insert(
        organization.id,
        vec![
            "artist:write",
            "event:interest",
            "event:view-guests",
            "event:write",
            "order:read",
            "org:read",
            "org:write",
            "ticket:admin",
            "user:read",
            "venue:write",
        ].into_iter()
        .map(|scope| scope.to_string())
        .collect(),
    );
    assert_eq!(expected_results, current_user.organization_scopes);

    let mut expected_roles = HashMap::new();
    expected_roles.insert(
        organization.id,
        vec!["OrgOwner".to_string(), "OrgMember".to_string()],
    );
    assert_eq!(expected_roles, current_user.organization_roles);
}

#[test]
fn current_user_organization_member() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database
        .create_organization_with_user(&user, false)
        .finish();
    let user = User::find(user.id, &database.connection).unwrap();
    let auth_user = support::create_auth_user_from_user(&user, Roles::OrgMember, &database);

    let response: HttpResponse =
        users::current_user((database.connection.clone().into(), auth_user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let current_user: CurrentUser = serde_json::from_str(&body).unwrap();
    let user = current_user.user;
    assert_eq!(user.id, user.id);
    assert_eq!(vec!["event:interest", "order:read"], current_user.scopes);
    let mut expected_scopes = HashMap::new();
    expected_scopes.insert(
        organization.id,
        vec![
            "artist:write",
            "event:interest",
            "event:view-guests",
            "event:write",
            "order:read",
            "org:read",
            "ticket:admin",
            "venue:write",
        ].into_iter()
        .map(|scope| scope.to_string())
        .collect(),
    );
    assert_eq!(expected_scopes, current_user.organization_scopes);

    let mut expected_roles = HashMap::new();
    expected_roles.insert(organization.id, vec!["OrgMember".to_string()]);
    assert_eq!(expected_roles, current_user.organization_roles);
}

#[test]
pub fn update_current_user() {
    let database = TestDatabase::new();
    let user = support::create_auth_user(Roles::User, &database);
    let email = "new-email@tari.com";
    let mut attributes: UserProfileAttributes = Default::default();
    attributes.email = Some(email.to_string());
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
    let user = support::create_auth_user(Roles::User, &database);
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

    let user = support::create_auth_user(Roles::User, &database);
    let mut attributes: UserProfileAttributes = Default::default();
    attributes.email = existing_user.email;
    let json = Json(attributes);

    let response: HttpResponse =
        users::update_current_user((database.connection.into(), json, user)).into();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}
