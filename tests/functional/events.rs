use bigneon_db::models::Roles;
use functional::base::events;

#[cfg(test)]
mod index_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        events::index(Roles::OrgMember);
    }
    #[test]
    fn index_guest() {
        events::index(Roles::Guest);
    }
    #[test]
    fn index_admin() {
        events::index(Roles::Admin);
    }
    #[test]
    fn index_user() {
        events::index(Roles::User);
    }
    #[test]
    fn index_org_owner() {
        events::index(Roles::OrgOwner);
    }
}

#[cfg(test)]
mod show_tests {
    use super::*;
    #[test]
    fn show_org_member() {
        events::show(Roles::OrgMember);
    }
    #[test]
    fn show_guest() {
        events::show(Roles::Guest);
    }
    #[test]
    fn show_admin() {
        events::show(Roles::Admin);
    }
    #[test]
    fn show_user() {
        events::show(Roles::User);
    }
    #[test]
    fn show_org_owner() {
        events::show(Roles::OrgOwner);
    }
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        events::create(Roles::OrgMember);
    }
    #[test]
    fn create_guest() {
        events::create(Roles::Guest);
    }
    #[test]
    fn create_admin() {
        events::create(Roles::Admin);
    }
    #[test]
    fn create_user() {
        events::create(Roles::User);
    }
    #[test]
    fn create_org_owner() {
        events::create(Roles::OrgOwner);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        events::update(Roles::OrgMember);
    }
    #[test]
    fn update_guest() {
        events::update(Roles::Guest);
    }
    #[test]
    fn update_admin() {
        events::update(Roles::Admin);
    }
    #[test]
    fn update_user() {
        events::update(Roles::User);
    }
    #[test]
    fn update_org_owner() {
        events::update(Roles::OrgOwner);
    }
}

#[cfg(test)]
mod show_from_organizations_tests {
    use super::*;
    #[test]
    fn show_from_organizations_org_member() {
        events::show_from_organizations(Roles::OrgMember);
    }
    #[test]
    fn show_from_organizations_guest() {
        events::show_from_organizations(Roles::Guest);
    }
    #[test]
    fn show_from_organizations_admin() {
        events::show_from_organizations(Roles::Admin);
    }
    #[test]
    fn show_from_organizations_user() {
        events::show_from_organizations(Roles::User);
    }
    #[test]
    fn show_from_organizations_org_owner() {
        events::show_from_organizations(Roles::OrgOwner);
    }
}

#[cfg(test)]
mod show_from_venues_tests {
    use super::*;
    #[test]
    fn show_from_venues_org_member() {
        events::show_from_venues(Roles::OrgMember);
    }
    #[test]
    fn show_from_venues_guest() {
        events::show_from_venues(Roles::Guest);
    }
    #[test]
    fn show_from_venues_admin() {
        events::show_from_venues(Roles::Admin);
    }
    #[test]
    fn show_from_venues_user() {
        events::show_from_venues(Roles::User);
    }
    #[test]
    fn show_from_venues_org_owner() {
        events::show_from_venues(Roles::OrgOwner);
    }
}
