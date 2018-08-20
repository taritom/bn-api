use bigneon_db::models::Roles;
use functional::base::users;

#[cfg(test)]
mod user_search_by_email_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        users::show_from_email(Roles::OrgMember, false);
    }
    #[test]
    fn index_guest() {
        users::show_from_email(Roles::Guest, false);
    }
    #[test]
    fn index_admin() {
        users::show_from_email(Roles::Admin, true);
    }
    #[test]
    fn index_user() {
        users::show_from_email(Roles::User, false);
    }
    #[test]
    fn index_org_owner() {
        users::show_from_email(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod users_show_tests {
    use super::*;
    #[test]
    fn show_org_member() {
        users::show(Roles::OrgMember, false);
    }
    #[test]
    fn show_guest() {
        users::show(Roles::Guest, false);
    }
    #[test]
    fn show_admin() {
        users::show(Roles::Admin, true);
    }
    #[test]
    fn show_user() {
        users::show(Roles::User, false);
    }
    #[test]
    fn show_org_owner() {
        users::show(Roles::OrgOwner, true);
    }
}
