use actix_web::{FromRequest, HttpResponse, Path};
use bigneon_api::controllers::organization_invites;
use bigneon_api::models::OrganizationInvitePathParameters;
use bigneon_db::models::*;
use functional::base;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::organization_invites::create(Roles::OrgMember, false);
    }
    #[test]
    fn create_admin() {
        base::organization_invites::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        base::organization_invites::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        base::organization_invites::create(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        base::organization_invites::index(Roles::OrgMember, false);
    }
    #[test]
    fn index_admin() {
        base::organization_invites::index(Roles::Admin, true);
    }
    #[test]
    fn index_user() {
        base::organization_invites::index(Roles::User, false);
    }
    #[test]
    fn index_org_owner() {
        base::organization_invites::index(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod destroy_tests {
    use super::*;
    #[test]
    fn destroy_org_member() {
        base::organization_invites::destroy(Roles::OrgMember, false);
    }
    #[test]
    fn destroy_admin() {
        base::organization_invites::destroy(Roles::Admin, true);
    }
    #[test]
    fn destroy_user() {
        base::organization_invites::destroy(Roles::User, false);
    }
    #[test]
    fn destroy_org_owner() {
        base::organization_invites::destroy(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod create_for_new_user_tests {
    use super::*;
    #[test]
    fn create_for_new_user_org_member() {
        base::organization_invites::create_for_new_user(Roles::OrgMember, false);
    }
    #[test]
    fn create_for_new_user_admin() {
        base::organization_invites::create_for_new_user(Roles::Admin, true);
    }
    #[test]
    fn create_for_new_user_user() {
        base::organization_invites::create_for_new_user(Roles::User, false);
    }
    #[test]
    fn create_for_new_user_org_owner() {
        base::organization_invites::create_for_new_user(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod accept_tests {
    use super::*;
    #[test]
    fn accept_org_member() {
        base::organization_invites::accept_invite_status_of_invite(Roles::OrgMember, true);
    }
    #[test]
    fn accept_admin() {
        base::organization_invites::accept_invite_status_of_invite(Roles::Admin, true);
    }
    #[test]
    fn accept_user() {
        base::organization_invites::accept_invite_status_of_invite(Roles::User, true);
    }
    #[test]
    fn accept_org_owner() {
        base::organization_invites::accept_invite_status_of_invite(Roles::OrgOwner, true);
    }
}

#[test]
fn destroy_owner_role_invite_as_organization_member_fails() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, Roles::OrgAdmin, Some(&organization), &database);

    let invite = database
        .create_organization_invite()
        .with_role(Roles::OrgOwner)
        .with_org(&organization)
        .with_invitee(&user)
        .finish();

    let test_request = TestRequest::create_with_uri_custom_params("/", vec!["id", "invite_id"]);
    let mut path =
        Path::<OrganizationInvitePathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    path.invite_id = invite.id;

    let response: HttpResponse =
        organization_invites::destroy((database.connection.clone().into(), path, auth_user)).into();

    support::expects_unauthorized(&response);
}
