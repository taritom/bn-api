use bigneon_db::models::Roles;
use functional::base::organizations;

#[cfg(test)]
mod index_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        organizations::index(Roles::OrgMember, true);
    }
    #[test]
    fn index_guest() {
        organizations::index(Roles::Guest, false);
    }
    #[test]
    fn index_admin() {
        organizations::index(Roles::Admin, true);
    }
    #[test]
    fn index_user() {
        organizations::index(Roles::User, false);
    }
    #[test]
    fn index_org_owner() {
        organizations::index(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod index_for_admin_tests {
    use super::*;
    #[test]
    fn index_for_admin_org_member() {
        organizations::index_for_admin(Roles::OrgMember, true);
    }
    #[test]
    fn index_for_admin_guest() {
        organizations::index_for_admin(Roles::Guest, false);
    }
    #[test]
    fn index_for_admin_admin() {
        organizations::index_for_admin(Roles::Admin, true);
    }
    #[test]
    fn index_for_admin_user() {
        organizations::index_for_admin(Roles::User, false);
    }
    #[test]
    fn index_for_admin_org_owner() {
        organizations::index_for_admin(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        organizations::create(Roles::OrgMember);
    }
    #[test]
    fn create_guest() {
        organizations::create(Roles::Guest);
    }
    #[test]
    fn create_admin() {
        organizations::create(Roles::Admin);
    }
    #[test]
    fn create_user() {
        organizations::create(Roles::User);
    }
    #[test]
    fn create_org_owner() {
        organizations::create(Roles::OrgOwner);
    }
}

#[cfg(test)]
mod remove_user_tests {
    use super::*;
    #[test]
    fn remove_user_org_member() {
        organizations::remove_user(Roles::OrgMember, false);
    }
    #[test]
    fn remove_user_guest() {
        organizations::remove_user(Roles::Guest, false);
    }
    #[test]
    fn remove_user_admin() {
        organizations::remove_user(Roles::Admin, false);
    }
    #[test]
    fn remove_user_user() {
        organizations::remove_user(Roles::User, false);
    }
    #[test]
    fn remove_user_org_owner() {
        organizations::remove_user(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        organizations::update(Roles::OrgMember, false);
    }
    #[test]
    fn update_guest() {
        organizations::update(Roles::Guest, false);
    }
    #[test]
    fn update_admin() {
        organizations::update(Roles::Admin, false);
    }
    #[test]
    fn update_user() {
        organizations::update(Roles::User, false);
    }
    #[test]
    fn update_org_owner() {
        organizations::update(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod update_owner_tests {
    use super::*;
    #[test]
    fn update_owner_org_member() {
        organizations::update_owner(Roles::OrgMember, false);
    }
    #[test]
    fn update_owner_guest() {
        organizations::update_owner(Roles::Guest, false);
    }
    #[test]
    fn update_owner_admin() {
        organizations::update_owner(Roles::Admin, true);
    }
    #[test]
    fn update_owner_user() {
        organizations::update_owner(Roles::User, false);
    }
    #[test]
    fn update_owner_org_owner() {
        organizations::update_owner(Roles::OrgOwner, false);
    }
}
