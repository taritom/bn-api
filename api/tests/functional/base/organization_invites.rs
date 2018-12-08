use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path, Query};
use bigneon_api::controllers::organization_invites::{self, *};
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let email = "jeff2@tari.com";
    let _invited_user = database
        .create_user()
        .with_email(email.to_string())
        .finish();

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let json = Json(NewOrgInviteRequest {
        user_email: Some(email.into()),
        user_id: None,
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse = organization_invites::create((
        state,
        database.connection.into(),
        json,
        path,
        auth_user.clone(),
    ))
    .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let org_in: OrganizationInvite = serde_json::from_str(&body).unwrap();
        assert_eq!(org_in.organization_id, organization.id);
        assert_eq!(org_in.inviter_id, user.id);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn create_for_existing_user_via_user_id(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let email = "test@tari.com";
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let invited_user = database
        .create_user()
        .with_email(email.to_string())
        .finish();

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let json = Json(NewOrgInviteRequest {
        user_email: None,
        user_id: Some(invited_user.id),
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse =
        organization_invites::create((state, database.connection.into(), json, path, auth_user))
            .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let org_in: OrganizationInvite = serde_json::from_str(&body).unwrap();
        assert_eq!(org_in.organization_id, organization.id);
        assert_eq!(org_in.inviter_id, user.id);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn create_for_new_user(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let email = "jeff2@tari.com";
    let json = Json(NewOrgInviteRequest {
        user_email: Some(email.into()),
        user_id: None,
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse =
        organization_invites::create((state, database.connection.into(), json, path, auth_user))
            .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
        let org_in: OrganizationInvite = serde_json::from_str(&body).unwrap();
        assert_eq!(org_in.organization_id, organization.id);
        assert_eq!(org_in.inviter_id, user.id);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn create_failure_missing_required_parameters(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let test_request = TestRequest::create();
    let state = test_request.extract_state();

    let json = Json(NewOrgInviteRequest {
        user_email: None,
        user_id: None,
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let response: HttpResponse =
        organization_invites::create((state, database.connection.into(), json, path, auth_user))
            .into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let expected_json =
            json!({"error": "Missing required parameters, `user_id` or `user_email` required"})
                .to_string();
        assert_eq!(body, expected_json);
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn accept_invite_status_of_invite(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    database.create_user().finish();

    let invite = database
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .with_email(&user.email.clone().unwrap())
        .with_security_token(None)
        .finish();

    OrganizationInvite::get_invite_details(
        &invite.security_token.unwrap(),
        database.connection.get(),
    )
    .unwrap();

    let test_request = TestRequest::create_with_uri(
        format!(
            "/accept_invite?security_token={}",
            &invite.security_token.unwrap().to_string()
        )
        .as_str(),
    );
    let parameters = Query::<InviteResponseQuery>::extract(&test_request.request).unwrap();

    let response: HttpResponse = organization_invites::accept_request((
        database.connection.into(),
        parameters,
        Some(auth_user),
        test_request.request,
    ))
    .into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        println!("{:?}", body);
    } else {
        support::expects_unauthorized(&response);
    }
}
pub fn decline_invite_status_of_invite(role: Roles, should_test_true: bool) {
    let database = TestDatabase::new();
    let email = "test@tari.com";
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    database
        .create_user()
        .with_email(email.to_string())
        .finish();
    let invite = database
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .with_security_token(None)
        .finish();

    OrganizationInvite::get_invite_details(
        &invite.security_token.unwrap(),
        database.connection.get(),
    )
    .unwrap();

    let test_request = TestRequest::create_with_uri(
        format!(
            "/decline_invite?security_token={}",
            &invite.security_token.unwrap().to_string()
        )
        .as_str(),
    );

    let parameters = Query::<InviteResponseQuery>::extract(&test_request.request).unwrap();

    let response: HttpResponse = organization_invites::decline_request((
        database.connection.into(),
        parameters,
        Some(auth_user),
    ))
    .into();

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        println!("{:?}", body);
    } else {
        support::expects_unauthorized(&response);
    }
}
