use bigneon_db::models::Roles;
use functional::base::organization_invites;

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        organization_invites::create(Roles::OrgMember, false, true);
    }
    #[test]
    fn create_admin() {
        organization_invites::create(Roles::Admin, true, true);
    }
    #[test]
    fn create_user() {
        organization_invites::create(Roles::User, false, true);
    }
    #[test]
    fn create_org_owner() {
        organization_invites::create(Roles::OrgOwner, true, true);
    }
    #[test]
    fn create_other_organization_org_member() {
        organization_invites::create(Roles::OrgMember, false, false);
    }
    #[test]
    fn create_other_organization_admin() {
        organization_invites::create(Roles::Admin, true, false);
    }
    #[test]
    fn create_other_organization_user() {
        organization_invites::create(Roles::User, false, false);
    }
    #[test]
    fn create_other_organization_org_owner() {
        organization_invites::create(Roles::OrgOwner, false, false);
    }
}

#[cfg(test)]
mod create_failure_missing_required_parameters_tests {
    use super::*;
    #[test]
    fn create_failure_missing_required_parameters_org_member() {
        organization_invites::create_failure_missing_required_parameters(Roles::OrgMember, false);
    }
    #[test]
    fn create_failure_missing_required_parameters_admin() {
        organization_invites::create_failure_missing_required_parameters(Roles::Admin, true);
    }
    #[test]
    fn create_failure_missing_required_parameters_user() {
        organization_invites::create_failure_missing_required_parameters(Roles::User, false);
    }
    #[test]
    fn create_failure_missing_required_parameters_org_owner() {
        organization_invites::create_failure_missing_required_parameters(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod create_for_existing_user_via_user_id_tests {
    use super::*;
    #[test]
    fn create_for_existing_user_via_user_id_org_member() {
        organization_invites::create_for_existing_user_via_user_id(Roles::OrgMember, false);
    }
    #[test]
    fn create_for_existing_user_via_user_id_admin() {
        organization_invites::create_for_existing_user_via_user_id(Roles::Admin, true);
    }
    #[test]
    fn create_for_existing_user_via_user_id_user() {
        organization_invites::create_for_existing_user_via_user_id(Roles::User, false);
    }
    #[test]
    fn create_for_existing_user_via_user_id_org_owner() {
        organization_invites::create_for_existing_user_via_user_id(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod create_for_new_user_tests {
    use super::*;
    #[test]
    fn create_for_new_user_org_member() {
        organization_invites::create_for_new_user(Roles::OrgMember, false);
    }
    #[test]
    fn create_for_new_user_admin() {
        organization_invites::create_for_new_user(Roles::Admin, true);
    }
    #[test]
    fn create_for_new_user_user() {
        organization_invites::create_for_new_user(Roles::User, false);
    }
    #[test]
    fn create_for_new_user_org_owner() {
        organization_invites::create_for_new_user(Roles::OrgOwner, true);
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
