use bigneon_db::models::Roles;
use functional::base::venues;

#[cfg(test)]
mod index_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        venues::index(Roles::OrgMember, true);
    }
    #[test]
    fn index_guest() {
        venues::index(Roles::Guest, true);
    }
    #[test]
    fn index_admin() {
        venues::index(Roles::Admin, true);
    }
    #[test]
    fn index_user() {
        venues::index(Roles::User, true);
    }
    #[test]
    fn index_org_owner() {
        venues::index(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod show_tests {
    use super::*;
    #[test]
    fn show_org_member() {
        venues::show(Roles::OrgMember, true);
    }
    #[test]
    fn show_guest() {
        venues::show(Roles::Guest, true);
    }
    #[test]
    fn show_admin() {
        venues::show(Roles::Admin, true);
    }
    #[test]
    fn show_user() {
        venues::show(Roles::User, true);
    }
    #[test]
    fn show_org_owner() {
        venues::show(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        venues::create(Roles::OrgMember, false);
    }
    #[test]
    fn create_guest() {
        venues::create(Roles::Guest, false);
    }
    #[test]
    fn create_admin() {
        venues::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        venues::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        venues::create(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        venues::update(Roles::OrgMember, false);
    }
    #[test]
    fn update_guest() {
        venues::update(Roles::Guest, false);
    }
    #[test]
    fn update_admin() {
        venues::update(Roles::Admin, true);
    }
    #[test]
    fn update_user() {
        venues::update(Roles::User, false);
    }
    #[test]
    fn update_org_owner() {
        venues::update(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod show_from_organizations_tests {
    use super::*;
    #[test]
    fn show_from_organizations_org_member() {
        venues::show_from_organizations(Roles::OrgMember, true);
    }
    #[test]
    fn show_from_organizations_guest() {
        venues::show_from_organizations(Roles::Guest, true);
    }
    #[test]
    fn show_from_organizations_admin() {
        venues::show_from_organizations(Roles::Admin, true);
    }
    #[test]
    fn show_from_organizations_user() {
        venues::show_from_organizations(Roles::User, true);
    }
    #[test]
    fn show_from_organizations_org_owner() {
        venues::show_from_organizations(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod add_to_organization_tests {
    use super::*;
    #[test]
    fn add_to_organization_org_member() {
        venues::add_to_organization(Roles::OrgMember, false);
    }
    #[test]
    fn add_to_organization_guest() {
        venues::add_to_organization(Roles::Guest, false);
    }
    #[test]
    fn add_to_organization_admin() {
        venues::add_to_organization(Roles::Admin, true);
    }
    #[test]
    fn add_to_organization_user() {
        venues::add_to_organization(Roles::User, false);
    }
    #[test]
    fn add_to_organization_org_owner() {
        venues::add_to_organization(Roles::OrgOwner, false);
    }
}
