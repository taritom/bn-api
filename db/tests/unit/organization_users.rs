use bigneon_db::dev::TestProject;
use bigneon_db::models::{OrganizationUser, Roles};

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_member(&user, Roles::OrgOwner)
        .finish();
    let organization_user =
        OrganizationUser::create(organization.id, user2.id, vec![Roles::OrgMember])
            .commit(project.get_connection())
            .unwrap();

    assert_eq!(organization_user.user_id, user2.id);
    assert_eq!(organization_user.organization_id, organization.id);
    assert_eq!(organization_user.role, [Roles::OrgMember]);
    assert_eq!(organization_user.id.to_string().is_empty(), false);
}
