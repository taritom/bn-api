use bigneon_db::models::Roles;
use functional::base::organizations;

#[cfg(test)]
mod index_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        organizations::index(Roles::OrgMember);
    }
    #[test]
    fn index_guest() {
        organizations::index(Roles::Guest);
    }
    #[test]
    fn index_admin() {
        organizations::index(Roles::Admin);
    }
    #[test]
    fn index_user() {
        organizations::index(Roles::User);
    }
    #[test]
    fn index_org_owner() {
        organizations::index(Roles::OrgOwner);
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
