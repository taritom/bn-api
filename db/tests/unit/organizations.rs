use bigneon_db::models::{Organization, OrganizationEditableAttributes, OrganizationUser};
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
    edited_organization.zip = Some("0124".to_string());
    edited_organization.phone = Some("+27123456789".to_string());

    let mut changed_attrs: OrganizationEditableAttributes = Default::default();
    changed_attrs.name = Some("Test Org".to_string());
    changed_attrs.address = Some("Test Address".to_string());
    changed_attrs.city = Some("Test Address".to_string());
    changed_attrs.state = Some("Test state".to_string());
    changed_attrs.country = Some("Test country".to_string());
    changed_attrs.zip = Some("0124".to_string());
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
    let db_org = Organization::find(&organization.id, &project).unwrap();
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
    let found_organization = Organization::find(&organization.id, &project).unwrap();
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
    let user_results = organization.users(&project);
    assert_eq!(user_results.len(), 1);
    assert_eq!(user.id, user_results[0].id);

    let user_results = organization2.users(&project);
    assert_eq!(
        vec![user3.id, user2.id],
        user_results.iter().map(|u| u.id).collect::<Vec<Uuid>>()
    );

    // Explicitly make the organization user an org user
    OrganizationUser::create(organization.id, user.id)
        .commit(&project)
        .unwrap();
    let user_results = organization.users(&project);
    assert_eq!(user_results.len(), 1);
    assert_eq!(user.id, user_results[0].id);
    let user_results2 = organization2.users(&project);
    assert_eq!(user_results2.len(), 2);
    assert_eq!(user3.id, user_results2[0].id);
    assert_eq!(user2.id, user_results2[1].id);

    // Add a new user to the organization
    OrganizationUser::create(organization.id, user2.id)
        .commit(&project)
        .unwrap();
    let user_results = organization.users(&project);
    assert!(user_results.len() == 2);
    assert_eq!(user.id, user_results[0].id);
    assert_eq!(user2.id, user_results[1].id);
    let user_results2 = organization2.users(&project);
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
    let test_vec = vec![org1, org2];
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
    let test_vec = vec![org1, org2, org3];
    assert_eq!(orgs, test_vec);
}

#[test]
fn remove_users() {
    let project = TestProject::new();
    let user = project.create_user().finish();
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

    let user_results = organization.users(&project);
    let users_predelete = vec![user.clone(), user2, user3.clone()];
    assert_eq!(user_results, users_predelete);

    //remove user
    let result = organization.remove_user(&user2_id, &project).unwrap();
    assert_eq!(result, 1);
    let user_results2 = organization.users(&project);
    let users_postdelete = vec![user, user3];
    assert_eq!(user_results2, users_postdelete);
}
