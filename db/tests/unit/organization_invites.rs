extern crate chrono;
use bigneon_db::dev::TestProject;
use bigneon_db::models::OrganizationInvite;
use bigneon_db::utils::errors::ErrorCode::ValidationError;
use bigneon_db::utils::errors::{DatabaseError, ErrorCode};
use diesel;
use diesel::prelude::*;
use unit::organization_invites::chrono::prelude::*;

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let org_invite = project
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .finish();

    assert_eq!(org_invite.organization_id, organization.id);
    assert_eq!(org_invite.inviter_id, user.id);
}

#[test]
fn create_with_validation_errors() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let result = OrganizationInvite::create(
        organization.id,
        user.id,
        &"invalid-email".to_string(),
        Some(user.id),
    ).commit(project.get_connection());

    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("user_email"));
                assert_eq!(errors["user_email"].len(), 1);
                assert_eq!(errors["user_email"][0].code, "email");
                assert_eq!(
                    &errors["user_email"][0]
                        .message
                        .clone()
                        .unwrap()
                        .into_owned(),
                    "User email is invalid"
                );
            }
            _ => panic!("Expected validation error"),
        },
    }
}

#[test]
fn change_invite_status_of_invite() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let org_invite = project
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .link_to_user(&user2)
        .finish();
    let org_invite2 = project
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .link_to_user(&user2)
        .finish();
    /*making the assumption that it wont take more than 60 seconds to update the status
    we cant test for an exact date, as this will depend on the database write delay
    we will test for a period of 30 seconds
    */
    let compare_true = org_invite.accept_invite(&project.get_connection()).unwrap();
    let compare_false = org_invite2
        .decline_invite(&project.get_connection())
        .unwrap();

    assert_eq!(compare_true.accepted, Some(1));
    assert_eq!(compare_true.security_token, None);

    assert_eq!(compare_false.accepted, Some(0));
    assert_eq!(compare_false.security_token, None);
}

#[test]
fn view_invitation() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let inviter = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let org_invite = project
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .with_inviter(&inviter)
        .finish();
    let display_invite = OrganizationInvite::get_invite_display(
        &org_invite.security_token.unwrap(),
        project.get_connection(),
    ).unwrap();

    assert_eq!(display_invite.organization_name, organization.name);
    assert_eq!(
        display_invite.inviter_name,
        format!(
            "{} {}",
            inviter.first_name.unwrap_or("".to_string()),
            inviter.last_name.unwrap_or("".to_string())
        )
    );
}
#[test]
fn test_token_validity() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let mut org_invite = project
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .finish();
    let recovered_invite = OrganizationInvite::get_invite_details(
        &org_invite.security_token.unwrap(),
        project.get_connection(),
    ).unwrap();
    assert_eq!(org_invite, recovered_invite);
    org_invite.created_at = NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11);
    org_invite = update(&org_invite, &project.get_connection()).unwrap();
    let recovered_invite2 = OrganizationInvite::get_invite_details(
        &org_invite.security_token.unwrap(),
        &project.get_connection(),
    );
    let error_value = DatabaseError {
        code: 2000,
        message: "No results".into(),
        cause: Some("No valid token found, NotFound".into()),
        error_code: ErrorCode::NoResults,
    };
    match recovered_invite2 {
        Ok(_val) => assert_eq!(true, false), //this should not happen, so this should fail
        Err(val) => assert_eq!(error_value, val),
    }
}

#[test]
fn test_sending_status() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let org_invite = project
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .link_to_user(&user2)
        .finish();
    /*making the assumption that it wont take more than 60 seconds to update the status
    we cant test for an exact date, as this will depend on the database write delay
    we will test for a period of 30 seconds
    */
    let pre_send_invite = org_invite.clone();
    let post_send_invite = org_invite
        .change_sent_status(true, &project.get_connection())
        .unwrap();

    assert_eq!(pre_send_invite.sent_invite, false);
    assert_eq!(post_send_invite.sent_invite, true);
}

// dont want to update the details in the main function, so keeping this in the unit test section
fn update(
    org_invite: &OrganizationInvite,
    conn: &PgConnection,
) -> Result<OrganizationInvite, DatabaseError> {
    DatabaseError::wrap(
        ErrorCode::UpdateError,
        "Could not update organization_invite",
        diesel::update(org_invite).set(org_invite).get_result(conn),
    )
}
