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
mod show_tests {
    use super::*;
    #[test]
    fn show_org_member() {
        organizations::show(Roles::OrgMember, true);
    }
    #[test]
    fn show_admin() {
        organizations::show(Roles::Admin, true);
    }
    #[test]
    fn show_user() {
        organizations::show(Roles::User, false);
    }
    #[test]
    fn show_org_owner() {
        organizations::show(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod index_for_all_orgs_tests {
    use super::*;
    #[test]
    fn index_for_all_orgs_org_member() {
        organizations::index_for_all_orgs(Roles::OrgMember, false);
    }
    #[test]
    fn index_for_all_orgs_admin() {
        organizations::index_for_all_orgs(Roles::Admin, true);
    }
    #[test]
    fn index_for_all_orgs_user() {
        organizations::index_for_all_orgs(Roles::User, false);
    }
    #[test]
    fn index_for_all_orgs_org_owner() {
        organizations::index_for_all_orgs(Roles::OrgOwner, false);
    }
}

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        organizations::create(Roles::OrgMember, false);
    }
    #[test]
    fn create_admin() {
        organizations::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        organizations::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        organizations::create(Roles::OrgOwner, false);
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
    fn remove_user_admin() {
        organizations::remove_user(Roles::Admin, true);
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
mod add_user_tests {
    use super::*;
    #[test]
    fn add_user_org_member() {
        organizations::add_user(Roles::OrgMember, false);
    }
    #[test]
    fn add_user_admin() {
        organizations::add_user(Roles::Admin, true);
    }
    #[test]
    fn add_user_user() {
        organizations::add_user(Roles::User, false);
    }
    #[test]
    fn add_user_org_owner() {
        organizations::add_user(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod add_venue_tests {
    use super::*;
    #[test]
    fn add_venue_org_member() {
        organizations::add_venue(Roles::OrgMember, false);
    }
    #[test]
    fn add_venue_admin() {
        organizations::add_venue(Roles::Admin, true);
    }
    #[test]
    fn add_venue_user() {
        organizations::add_venue(Roles::User, false);
    }
    #[test]
    fn add_venue_org_owner() {
        organizations::add_venue(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod add_artist_tests {
    use super::*;
    #[test]
    fn add_artist_org_member() {
        organizations::add_artist(Roles::OrgMember, false);
    }
    #[test]
    fn add_artist_admin() {
        organizations::add_artist(Roles::Admin, true);
    }
    #[test]
    fn add_artist_user() {
        organizations::add_artist(Roles::User, false);
    }
    #[test]
    fn add_artist_org_owner() {
        organizations::add_artist(Roles::OrgOwner, true);
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
    fn update_admin() {
        organizations::update(Roles::Admin, true);
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

#[cfg(test)]
mod list_organization_members_tests {
    use super::*;
    #[test]
    fn list_organization_members_org_member() {
        organizations::list_organization_members(Roles::OrgMember, true);
    }
    #[test]
    fn list_organization_members_admin() {
        organizations::list_organization_members(Roles::Admin, true);
    }
    #[test]
    fn list_organization_members_user() {
        organizations::list_organization_members(Roles::User, false);
    }
    #[test]
    fn list_organization_members_org_owner() {
        organizations::list_organization_members(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod show_fee_schedule_tests {
    use super::*;
    #[test]
    fn show_fee_schedule_org_member() {
        organizations::show_fee_schedule(Roles::OrgMember, false);
    }
    #[test]
    fn show_fee_schedule_admin() {
        organizations::show_fee_schedule(Roles::Admin, true);
    }
    #[test]
    fn show_fee_schedule_user() {
        organizations::show_fee_schedule(Roles::User, false);
    }
    #[test]
    fn show_fee_schedule_org_owner() {
        organizations::show_fee_schedule(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod add_fee_schedule_tests {
    use super::*;
    #[test]
    fn add_fee_schedule_org_member() {
        organizations::add_fee_schedule(Roles::OrgMember, false);
    }
    #[test]
    fn add_fee_schedule_admin() {
        organizations::add_fee_schedule(Roles::Admin, true);
    }
    #[test]
    fn add_fee_schedule_user() {
        organizations::add_fee_schedule(Roles::User, false);
    }
    #[test]
    fn add_fee_schedule_org_owner() {
        organizations::add_fee_schedule(Roles::OrgOwner, false);
    }
}
