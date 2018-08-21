use bigneon_db::models::Roles;
use functional::base::organization_invites;

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        organization_invites::create(Roles::OrgMember, false);
    }
    #[test]
    fn create_guest() {
        organization_invites::create(Roles::Guest, false);
    }
    #[test]
    fn create_admin() {
        organization_invites::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        organization_invites::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        organization_invites::create(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod accept_tests {
    use super::*;
    #[test]
    fn accept_org_member() {
        organization_invites::accept_invite_status_of_invite(Roles::OrgMember, true);
    }
    #[test]
    fn accept_guest() {
        organization_invites::accept_invite_status_of_invite(Roles::Guest, true);
    }
    #[test]
    fn accept_admin() {
        organization_invites::accept_invite_status_of_invite(Roles::Admin, true);
    }
    #[test]
    fn accept_user() {
        organization_invites::accept_invite_status_of_invite(Roles::User, true);
    }
    #[test]
    fn accept_org_owner() {
        organization_invites::accept_invite_status_of_invite(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod decline_tests {
    use super::*;
    #[test]
    fn decline_org_member() {
        organization_invites::decline_invite_status_of_invite(Roles::OrgMember, true);
    }
    #[test]
    fn decline_guest() {
        organization_invites::decline_invite_status_of_invite(Roles::Guest, true);
    }
    #[test]
    fn decline_admin() {
        organization_invites::decline_invite_status_of_invite(Roles::Admin, true);
    }
    #[test]
    fn decline_user() {
        organization_invites::decline_invite_status_of_invite(Roles::User, true);
    }
    #[test]
    fn decline_org_owner() {
        organization_invites::decline_invite_status_of_invite(Roles::OrgOwner, true);
    }

}

#[cfg(test)]
mod send_organization_invite_email_tests {
    use super::*;
    #[test]
    fn send_mail() {
        organization_invites::test_email();
    }
    // TODO: Test negative case
}
