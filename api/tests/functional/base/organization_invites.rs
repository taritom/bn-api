use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::organization_invites::{
    self, Info, NewOrgInviteRequest, PathParameters,
};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{OrganizationInvite, Roles};
use lettre::SendableEmail;
use serde_json;
use std::str;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let owner = database.create_user().finish();
    let organization = database.create_organization().with_owner(&owner).finish();

    let email = "jeff2@tari.com";
    let invited_user = database
        .create_user()
        .with_email(email.to_string())
        .finish();

    let user = support::create_auth_user_from_user(&owner, role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(NewOrgInviteRequest {
        user_email: Some(email.into()),
        user_id: None,
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse = organization_invites::create((state, json, path, user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let org_in: OrganizationInvite = serde_json::from_str(&body).unwrap();
        assert_eq!(org_in.organization_id, organization.id);
        assert_eq!(org_in.inviter_id, owner.id);

        let mail_transport = test_request.test_transport();

        {
            let sent = mail_transport.sent.lock().unwrap();
            let mail = sent.first().expect("An invite mail was expected");
            let envelope = mail.envelope();
            let email_body = str::from_utf8(*mail.message()).unwrap();
            assert_eq!(
                format!("{:?}", envelope.to()),
                format!("[EmailAddress(\"{}\")]", email)
            );
            assert_eq!(
                format!("{:?}", envelope.from().unwrap()),
                "EmailAddress(\"support@bigneon.com\")"
            );

            assert!(email_body.contains(&format!(
                "Hi {} {}",
                invited_user.first_name, invited_user.last_name
            )));
            assert!(email_body.contains("This invite link is valid for 7 days."));
            assert!(email_body.contains(org_in.security_token.unwrap().to_string().as_str()));
        }
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

pub fn create_for_existing_user_via_user_id(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let email = "test@tari.com";
    let owner = database.create_user().finish();
    let organization = database.create_organization().with_owner(&owner).finish();
    let invited_user = database
        .create_user()
        .with_email(email.to_string())
        .finish();

    let auth_user = support::create_auth_user_from_user(&owner, role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(NewOrgInviteRequest {
        user_email: None,
        user_id: Some(invited_user.id),
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse =
        organization_invites::create((state, json, path, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let org_in: OrganizationInvite = serde_json::from_str(&body).unwrap();
        assert_eq!(org_in.organization_id, organization.id);
        assert_eq!(org_in.inviter_id, owner.id);

        let mail_transport = test_request.test_transport();

        {
            let sent = mail_transport.sent.lock().unwrap();
            let mail = sent.first().expect("An invite mail was expected");
            let envelope = mail.envelope();
            let email_body = str::from_utf8(*mail.message()).unwrap();
            assert_eq!(
                format!("{:?}", envelope.to()),
                format!("[EmailAddress(\"{}\")]", email)
            );
            assert_eq!(
                format!("{:?}", envelope.from().unwrap()),
                "EmailAddress(\"support@bigneon.com\")"
            );

            assert!(email_body.contains(&format!(
                "Hi {} {}",
                invited_user.first_name, invited_user.last_name
            )));
            assert!(email_body.contains("This invite link is valid for 7 days."));
            assert!(email_body.contains(org_in.security_token.unwrap().to_string().as_str()));
        }
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

pub fn create_for_new_user(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let owner = database.create_user().finish();
    let organization = database.create_organization().with_owner(&owner).finish();
    let auth_user = support::create_auth_user_from_user(&owner, role, &database);

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let email = "jeff2@tari.com";
    let json = Json(NewOrgInviteRequest {
        user_email: Some(email.into()),
        user_id: None,
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse =
        organization_invites::create((state, json, path, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let org_in: OrganizationInvite = serde_json::from_str(&body).unwrap();
        assert_eq!(org_in.organization_id, organization.id);
        assert_eq!(org_in.inviter_id, owner.id);

        let mail_transport = test_request.test_transport();

        {
            let sent = mail_transport.sent.lock().unwrap();
            let mail = sent.first().expect("An invite mail was expected");
            let envelope = mail.envelope();
            let email_body = str::from_utf8(*mail.message()).unwrap();

            assert_eq!(
                format!("{:?}", envelope.to()),
                format!("[EmailAddress(\"{}\")]", email)
            );
            assert_eq!(
                format!("{:?}", envelope.from().unwrap()),
                "EmailAddress(\"support@bigneon.com\")"
            );

            assert!(email_body.contains("Hi New user"));
            assert!(email_body.contains("This invite link is valid for 7 days."));
            assert!(email_body.contains(org_in.security_token.unwrap().to_string().as_str()));
        }
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let organization_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, organization_expected_json);
    }
}

pub fn create_failure_missing_required_parameters(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let owner = database.create_user().finish();
    let organization = database.create_organization().with_owner(&owner).finish();
    let auth_user = support::create_auth_user_from_user(&owner, role, &database);

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let json = Json(NewOrgInviteRequest {
        user_email: None,
        user_id: None,
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse =
        organization_invites::create((state, json, path, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let expected_json = json!({"error": "Missing required parameters, `user_id` or `user_email` required"})
            .to_string();
        assert_eq!(body, expected_json);

        let mail_transport = test_request.test_transport();

        {
            let sent = mail_transport.sent.lock().unwrap();
            assert!(sent.first().is_none());
        }
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
    let owner = database.create_user().finish();
    let organization = database.create_organization().with_owner(&owner).finish();
    let invited_user = database.create_user().finish();

    let invite = database
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&owner)
        .with_security_token(None)
        .finish();

    let auth_user = support::create_auth_user_from_user(&owner, role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let org_invite =
        OrganizationInvite::get_invite_details(&invite.security_token.unwrap(), &*connection)
            .unwrap();

    let json = Json(Info {
        token: org_invite.security_token.unwrap(),
        user_id: invited_user.id,
    });

    let response: HttpResponse =
        organization_invites::accept_request((state, json, auth_user)).into();
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
    let email = "test@tari.com";
    let owner = database.create_user().finish();
    let organization = database.create_organization().with_owner(&owner).finish();
    let new_member = database
        .create_user()
        .with_email(email.to_string())
        .finish();
    let invite = database
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&owner)
        .with_security_token(None)
        .finish();

    let auth_user = support::create_auth_user_from_user(&owner, role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let org_invite =
        OrganizationInvite::get_invite_details(&invite.security_token.unwrap(), &*connection)
            .unwrap();

    let json = Json(Info {
        token: org_invite.security_token.unwrap(),
        user_id: new_member.id,
    });
    let response: HttpResponse =
        organization_invites::decline_request((state, json, auth_user)).into();

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
