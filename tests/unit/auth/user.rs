use bigneon_api::auth::user::User;
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::Roles;
use bigneon_db::models::User as DbUser;
use support::database::TestDatabase;

#[test]
fn is_in_role() {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let user = DbUser::create("Jeff", "Last", "test@test.com", "555-555-5555", "password")
        .commit(&*connection)
        .unwrap();

    let user = user.add_role(Roles::Guest, &*connection).unwrap();
    let user = user.add_role(Roles::Admin, &*connection).unwrap();

    let u = User::new(user);
    assert_eq!(u.is_in_role(Roles::Guest), true);
    assert_eq!(u.is_in_role(Roles::Admin), true);
    assert_eq!(u.is_in_role(Roles::OrgMember), false);
}

#[test]
fn requires_role() {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let user = DbUser::create("Jeff", "Last", "test@test.com", "555-555-5555", "password")
        .commit(&*connection)
        .unwrap();

    let user = user.add_role(Roles::Guest, &*connection).unwrap();
    let user = user.add_role(Roles::Admin, &*connection).unwrap();

    let u = User::new(user);
    assert!(u.requires_role(Roles::Guest).is_ok());
    assert!(u.requires_role(Roles::Admin).is_ok());
    assert!(u.requires_role(Roles::OrgMember).is_err());
}
