extern crate chrono;
use bigneon_db::db::Connectable;
use bigneon_db::models::OrganizationInvite;
use bigneon_db::utils::errors::{DatabaseError, ErrorCode};
use chrono::{Duration, Utc};
use diesel;
use diesel::prelude::*;
use support::project::TestProject;
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
fn add_user_to_invite() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let org_invite = project
        .create_organization_invite()
        .with_org(&organization)
        .with_invitee(&user)
        .finish();
    let updated_invite = org_invite.add_user_id(&user2.id, &project).unwrap();
    assert_eq!(updated_invite.user_id.unwrap(), user2.id);

    let _updated_invite_done =
        OrganizationInvite::get_invite_details(&org_invite.security_token.unwrap(), &project)
            .unwrap();

    assert_eq!(updated_invite.user_id.unwrap(), user2.id);
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
    /*making the asumption that it wont take more than 60 seconds to update the status
    we cant test for an exact date, as this will depend on the database write delay
    we will test for a period of 30 seconds
    */
    let testdate_start = Utc::now().naive_utc();
    let testdate_end = Utc::now().naive_utc() + Duration::seconds(60);
    let compare_true = org_invite.acepted_invite(&project).unwrap();
    let compare_false = org_invite2.decline_invite(&project).unwrap();

    assert_eq!(compare_true.accepted, Some(1));
    assert_eq!(compare_true.security_token, None);
    assert_eq!(
        compare_true.status_change_at.unwrap() > testdate_start,
        true
    );
    assert_eq!(compare_true.status_change_at.unwrap() < testdate_end, true);

    assert_eq!(compare_false.accepted, Some(0));
    assert_eq!(compare_false.security_token, None);
    assert_eq!(
        compare_false.status_change_at.unwrap() > testdate_start,
        true
    );
    assert_eq!(compare_false.status_change_at.unwrap() < testdate_end, true);
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
    let recovered_invite =
        OrganizationInvite::get_invite_details(&org_invite.security_token.unwrap(), &project)
            .unwrap();
    assert_eq!(org_invite, recovered_invite);
    org_invite.create_at = NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11);
    org_invite = update(&org_invite, &project).unwrap();
    let recovered_invite2 =
        OrganizationInvite::get_invite_details(&org_invite.security_token.unwrap(), &project);
    let error_value = DatabaseError {
        code: 6000,
        message: "Access error".into(),
        cause: Some("No valid token found, NotFound".into()),
    };
    match recovered_invite2 {
        Ok(_val) => assert_eq!(true, false), //this should not happen, so this should fail
        Err(val) => assert_eq!(error_value, val),
    }
}

// dont want to update the details in the main function, so keeping this in the unit test section
fn update(
    org_invite: &OrganizationInvite,
    conn: &Connectable,
) -> Result<OrganizationInvite, DatabaseError> {
    DatabaseError::wrap(
        ErrorCode::UpdateError,
        "Could not update organization_invite",
        diesel::update(org_invite)
            .set(org_invite)
            .get_result(conn.get_connection()),
    )
}
