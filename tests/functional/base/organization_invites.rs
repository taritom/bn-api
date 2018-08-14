use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::organization_invites::{self, Info, NewOrgInviteRequest};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{NewOrganizationInvite, Organization, OrganizationInvite, Roles, User};
use lettre::SendableEmail;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let user1 = User::create(
        "Jeff",
        "Roen",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let organization = Organization::create(user1.id, &"Organization")
        .commit(&*connection)
        .unwrap();

    let user2 = User::create(
        "Jeff2",
        "Wilco",
        "jeff2@tari.com",
        "555-555-55556",
        "examplePassword6",
    ).commit(&*connection)
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(NewOrgInviteRequest {
        organization_id: organization.id,
        user_email: "jeff2@tari.com".into(),
        user_id: None,
    });

    let mut user = support::create_auth_user(role, &*connection);
    user.user.id = user1.id;
    let response = organization_invites::create((state, json, user));
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let org_in: OrganizationInvite = serde_json::from_str(&body).unwrap();
        assert_eq!(org_in.organization_id, organization.id);
        assert_eq!(org_in.inviter_id, user1.id);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

pub fn accept_invite_status_of_invite(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let user1 = User::create(
        "Jeff",
        "Roen",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let organization = Organization::create(user1.id, &"Organization")
        .commit(&*connection)
        .unwrap();
    let user2 = User::create(
        "Jeff2",
        "Roen",
        "jeff2@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut new_invite = NewOrganizationInvite {
        organization_id: organization.id,
        inviter_id: user1.id,
        user_email: "jeff2@tari.com".into(),
        security_token: None,
        user_id: None,
    };

    let new_invite = new_invite.commit(&*connection).unwrap();

    let org_invite =
        OrganizationInvite::get_invite_details(&new_invite.security_token.unwrap(), &*connection)
            .unwrap();

    let user = support::create_auth_user(role, &*connection);
    let json = Json(Info {
        token: org_invite.security_token.unwrap(),
        user_id: user2.id,
    });
    let response = organization_invites::accept_request((state, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        println!("{:?}", body);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}
pub fn decline_invite_status_of_invite(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let user1 = User::create(
        "Jeff",
        "Roen",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let organization = Organization::create(user1.id, &"Organization")
        .commit(&*connection)
        .unwrap();
    let user2 = User::create(
        "Jeff2",
        "Roen",
        "jeff2@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut new_invite = NewOrganizationInvite {
        organization_id: organization.id,
        inviter_id: user1.id,
        user_email: "jeff2@tari.com".into(),
        security_token: None,
        user_id: None,
    };

    let new_invite = new_invite.commit(&*connection).unwrap();

    let org_invite =
        OrganizationInvite::get_invite_details(&new_invite.security_token.unwrap(), &*connection)
            .unwrap();

    let user = support::create_auth_user(role, &*connection);
    let json = Json(Info {
        token: org_invite.security_token.unwrap(),
        user_id: user2.id,
    });
    let response = organization_invites::decline_request((state, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        println!("{:?}", body);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

pub fn test_email(role: Roles, _should_test_true: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let email = "test@tari.com";
    let user1 = User::create(
        &"Name",
        &"Last",
        &email,
        &"555-555-5555",
        &"examplePassword",
    ).commit(&*database.get_connection())
        .unwrap();
    let user2 = User::create(
        "Jeff2",
        "Roen",
        "jeff2@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let organization = Organization::create(user2.id, &"Organization")
        .commit(&*connection)
        .unwrap();

    let mut new_invite = NewOrganizationInvite {
        organization_id: organization.id,
        inviter_id: user1.id,
        user_email: email.into(),
        security_token: None,
        user_id: None,
    };

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let new_invite = new_invite.commit(&*connection).unwrap();

    let org_invite =
        OrganizationInvite::get_invite_details(&new_invite.security_token.unwrap(), &*connection)
            .unwrap();
    organization_invites::create_invite_email(&state, &*connection, &org_invite, false);

    let mail_transport = test_request.test_transport();
    {
        assert_eq!(0, 0); //todo find a way to test email without requiring smtp. Currently this only test for no panic
    }
}
