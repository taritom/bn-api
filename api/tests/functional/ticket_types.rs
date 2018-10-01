use bigneon_db::models::Roles;
use functional::base::ticket_types;

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        ticket_types::create(Roles::OrgMember, true);
    }
    #[test]
    fn create_admin() {
        ticket_types::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        ticket_types::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        ticket_types::create(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        ticket_types::update(Roles::OrgMember, true);
    }
    #[test]
    fn update_admin() {
        ticket_types::update(Roles::Admin, true);
    }
    #[test]
    fn update_user() {
        ticket_types::update(Roles::User, false);
    }
    #[test]
    fn update_org_owner() {
        ticket_types::update(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        ticket_types::index(Roles::OrgMember, true, true);
    }
    #[test]
    fn index_admin() {
        ticket_types::index(Roles::Admin, true, true);
    }
    #[test]
    fn index_user() {
        ticket_types::index(Roles::User, false, true);
    }
    #[test]
    fn index_org_owner() {
        ticket_types::index(Roles::OrgOwner, true, true);
    }
    #[test]
    fn index_other_organization_org_member() {
        ticket_types::index(Roles::OrgMember, false, false);
    }
    #[test]
    fn index_other_organization_admin() {
        ticket_types::index(Roles::Admin, true, false);
    }
    #[test]
    fn index_other_organization_user() {
        ticket_types::index(Roles::User, false, false);
    }
    #[test]
    fn index_other_organization_org_owner() {
        ticket_types::index(Roles::OrgOwner, false, false);
    }
}
