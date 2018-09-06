use actix_web::{http::StatusCode, HttpResponse, Json};
use bigneon_api::controllers::users;
use bigneon_api::models::register_request::RegisterRequest;
use bigneon_db::models::Roles;
use functional::base;
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

    assert_eq!(response.status(), StatusCode::OK);
}
