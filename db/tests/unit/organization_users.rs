use bigneon_db::dev::TestProject;
use bigneon_db::models::OrganizationUser;

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization_user = OrganizationUser::create(organization.id, user2.id)
        .commit(project.get_connection())
        .unwrap();

    assert_eq!(organization_user.user_id, user2.id);
    assert_eq!(organization_user.organization_id, organization.id);
    assert_eq!(organization_user.id.to_string().is_empty(), false);
}
