use bigneon_db::models::{
    Organization, OrganizationEditableAttributes, OrganizationUser, Roles, User,
};
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = Organization::create(user.id, "Organization")
        .commit(&project)
        .unwrap();

    assert_eq!(organization.owner_user_id, user.id);
    assert_eq!(organization.id.to_string().is_empty(), false);

    let user2 = User::find(user.id, &project).unwrap();
    assert_eq!(user2.role, vec!["User", "OrgOwner"]);
}

#[test]
fn update() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    //Edit Organization
    let mut edited_organization = project.create_organization().with_owner(&user).finish();

    edited_organization.name = "Test Org".to_string();
    edited_organization.address = Some("Test Address".to_string());
    edited_organization.city = Some("Test Address".to_string());
    edited_organization.state = Some("Test state".to_string());
    edited_organization.country = Some("Test country".to_string());
    edited_organization.postal_code = Some("0124".to_string());
    edited_organization.phone = Some("+27123456789".to_string());

    let mut changed_attrs: OrganizationEditableAttributes = Default::default();
    changed_attrs.name = Some("Test Org".to_string());
    changed_attrs.address = Some("Test Address".to_string());
    changed_attrs.city = Some("Test Address".to_string());
    changed_attrs.state = Some("Test state".to_string());
    changed_attrs.country = Some("Test country".to_string());
    changed_attrs.postal_code = Some("0124".to_string());
    changed_attrs.phone = Some("+27123456789".to_string());
    let updated_organization =
        Organization::update(&edited_organization, changed_attrs, &project).unwrap();
    assert_eq!(edited_organization, updated_organization);
}

#[test]
fn update_owner() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    //Edit Organization
    let organization = project.create_organization().with_owner(&user).finish();

    let user2 = project.create_user().finish();

    let updated_org = organization.set_owner(user2.id, &project).unwrap();
    let db_org = Organization::find(organization.id, &project).unwrap();
    assert_eq!(updated_org.owner_user_id, user2.id);
    assert_eq!(db_org.owner_user_id, user2.id);
}

#[test]
fn find() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    //Edit Organization
    let organization = project
        .create_organization()
        .with_owner(&user)
        .with_address()
        .finish();
    let found_organization = Organization::find(organization.id, &project).unwrap();
    assert_eq!(organization, found_organization);
}

#[test]
fn users() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let user3 = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization2 = project.create_organization().with_owner(&user3).finish();
    OrganizationUser::create(organization2.id, user2.id)
        .commit(&project)
        .unwrap();

    // Owner is included in the user results for organization2 but not organization2
    let user_results = organization.users(&project).unwrap();
    assert_eq!(user_results.len(), 1);
    assert_eq!(user.id, user_results[0].id);

    let user_results = organization2.users(&project).unwrap();
    assert_eq!(
        vec![user3.id, user2.id],
        user_results.iter().map(|u| u.id).collect::<Vec<Uuid>>()
    );

    // Explicitly make the organization user an org user
    OrganizationUser::create(organization.id, user.id)
        .commit(&project)
        .unwrap();
    let user_results = organization.users(&project).unwrap();
    assert_eq!(user_results.len(), 1);
    assert_eq!(user.id, user_results[0].id);
    let user_results2 = organization2.users(&project).unwrap();
    assert_eq!(user_results2.len(), 2);
    assert_eq!(user3.id, user_results2[0].id);
    assert_eq!(user2.id, user_results2[1].id);

    // Add a new user to the organization
    OrganizationUser::create(organization.id, user2.id)
        .commit(&project)
        .unwrap();
    let user_results = organization.users(&project).unwrap();
    assert!(user_results.len() == 2);
    assert_eq!(user.id, user_results[0].id);
    assert_eq!(user2.id, user_results[1].id);
    let user_results2 = organization2.users(&project).unwrap();
    assert!(user_results2.len() == 2);
    assert_eq!(user3.id, user_results2[0].id);
    assert_eq!(user2.id, user_results2[1].id);
}

#[test]
fn all_linked_to_user() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let org1 = project.create_organization().with_owner(&user).finish();
    let org2 = project
        .create_organization()
        .with_owner(&user2)
        .with_user(&user)
        .finish();
    let _org3 = project.create_organization().with_owner(&user2).finish();

    let orgs = Organization::all_linked_to_user(user.id, &project).unwrap();
    let mut test_vec = vec![org1, org2];
    test_vec.sort_by_key(|org| org.name.clone());
    assert_eq!(orgs, test_vec);
}

#[test]
fn all() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let org1 = project.create_organization().with_owner(&user).finish();
    let org2 = project
        .create_organization()
        .with_owner(&user2)
        .with_user(&user)
        .finish();
    let org3 = project.create_organization().with_owner(&user2).finish();

    let orgs = Organization::all(&project).unwrap();
    let mut test_vec = vec![org1, org2, org3];
    test_vec.sort_by_key(|org| org.name.clone());
    assert_eq!(orgs, test_vec);
}

#[test]
fn remove_users() {
    let project = TestProject::new();
    let mut user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let user3 = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    OrganizationUser::create(organization.id, user2.id)
        .commit(&project)
        .unwrap();
    OrganizationUser::create(organization.id, user3.id)
        .commit(&project)
        .unwrap();
    let user2_id = user2.id;

    user.role.push("OrgOwner".to_string());

    let user_results = organization.users(&project).unwrap().sort_by_key(|k| k.id);
    let users_before_delete = vec![user.clone(), user2, user3.clone()].sort_by_key(|k| k.id);

    assert_eq!(user_results, users_before_delete);

    //remove user
    let result = organization.remove_user(user2_id, &project).unwrap();
    assert_eq!(result, 1);
    let user_results2 = organization.users(&project).unwrap();
    let users_post_delete = vec![user, user3];

    assert_eq!(user_results2, users_post_delete);
}

#[test]
fn change_owner() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let organization = Organization::create(user.id, "Organization")
        .commit(&project)
        .unwrap();

    let user2 = project.create_user().finish();

    organization.set_owner(user2.id, &project).unwrap();

    let user1_check = User::find(user.id, &project).unwrap();
    let user2_check = User::find(user2.id, &project).unwrap();

    assert_eq!(user1_check.role, vec!["User"]);
    assert_eq!(user2_check.role, vec!["User", "OrgOwner"]);
}

#[test]
fn add_user() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization_user = organization.add_user(user2.id, &project).unwrap();

    assert_eq!(organization_user.user_id, user2.id);
    assert_eq!(organization_user.organization_id, organization.id);
    assert_eq!(organization_user.id.to_string().is_empty(), false);
    let user2 = User::find(user2.id, &project).unwrap();
    assert!(
        user2.has_role(Roles::OrgMember),
        "User does not have OrgMember role"
    );
}

#[test]
fn add_fee_schedule() {
    let db = TestProject::new();
    let organization = db.create_organization().finish();
    let fee_structure = db.create_fee_schedule().finish();
    organization.add_fee_schedule(&fee_structure, &db).unwrap();
    let organization = Organization::find(organization.id, &db).unwrap();
    assert_eq!(organization.fee_schedule_id.unwrap(), fee_structure.id);
}
