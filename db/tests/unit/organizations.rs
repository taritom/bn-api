use bigneon_db::dev::TestProject;
use bigneon_db::prelude::*;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let fee_schedule = FeeSchedule::create(
        format!("Zero fees",).into(),
        vec![NewFeeScheduleRange {
            min_price_in_cents: 0,
            company_fee_in_cents: 0,
            client_fee_in_cents: 0,
        }],
    )
    .commit(connection)
    .unwrap();
    let mut organization = Organization::create("Organization", fee_schedule.id);
    organization.sendgrid_api_key = Some("A_Test_Key".to_string());

    let mut organization = organization
        .commit(&"encryption_key".to_string(), connection)
        .unwrap();
    assert_eq!(organization.id.to_string().is_empty(), false);

    assert_ne!(
        organization.sendgrid_api_key,
        Some("A_Test_Key".to_string())
    );
    organization.decrypt(&"encryption_key".to_string()).unwrap();
    assert_eq!(
        organization.sendgrid_api_key,
        Some("A_Test_Key".to_string())
    );
}

#[test]
fn has_fan() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish())
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];

    // No relationship
    assert!(!organization.has_fan(&user, connection).unwrap());

    // User adds item to cart but does not checkout so no relationship
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: None,
        }],
        false,
        connection,
    )
    .unwrap();
    assert!(!organization.has_fan(&user, connection).unwrap());

    // User checks out so has a paid order so relationship exists
    assert_eq!(cart.calculate_total(connection).unwrap(), 1700);
    cart.add_external_payment(Some("test".to_string()), user.id, 1700, connection)
        .unwrap();
    assert_eq!(cart.status().unwrap(), OrderStatus::Paid);
    assert!(organization.has_fan(&user, connection).unwrap());
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
    edited_organization.sendgrid_api_key = Some("A_Test_Key".to_string());

    let mut changed_attrs: OrganizationEditableAttributes = Default::default();
    changed_attrs.name = Some("Test Org".to_string());
    changed_attrs.address = Some("Test Address".to_string());
    changed_attrs.city = Some("Test Address".to_string());
    changed_attrs.state = Some("Test state".to_string());
    changed_attrs.country = Some("Test country".to_string());
    changed_attrs.postal_code = Some("0124".to_string());
    changed_attrs.phone = Some("+27123456789".to_string());
    changed_attrs.sendgrid_api_key = Some(Some("A_Test_Key".to_string()));
    let mut updated_organization = Organization::update(
        &edited_organization,
        changed_attrs,
        &"encryption_key".to_string(),
        project.get_connection(),
    )
    .unwrap();

    updated_organization
        .decrypt(&"encryption_key".to_string())
        .unwrap();

    assert_eq!(edited_organization, updated_organization);
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
    let found_organization = Organization::find(organization.id, project.get_connection()).unwrap();
    assert_eq!(organization, found_organization);
}

#[test]
fn find_for_event() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    //Edit Organization
    let organization = project
        .create_organization()
        .with_owner(&user)
        .with_address()
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .finish();
    let found_organization =
        Organization::find_for_event(event.id, project.get_connection()).unwrap();
    assert_eq!(organization, found_organization);
}

#[test]
fn users() {
    let project = TestProject::new();
    let user = project.create_user().with_last_name("User1").finish();
    let user2 = project.create_user().with_last_name("User2").finish();
    let user3 = project.create_user().with_last_name("User3").finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization2 = project.create_organization().with_owner(&user3).finish();
    OrganizationUser::create(organization2.id, user2.id, vec![Roles::OrgMember])
        .commit(project.get_connection())
        .unwrap();

    // Owner is included in the user results for organization2 but not organization2
    let user_results = organization.users(project.get_connection()).unwrap();
    assert_eq!(user_results.len(), 1);
    assert_eq!(user.id, user_results[0].1.id);

    let user_results = organization2.users(project.get_connection()).unwrap();
    assert_eq!(
        vec![user2.id, user3.id],
        user_results.iter().map(|u| u.1.id).collect::<Vec<Uuid>>()
    );

    // Explicitly make the organization user an org user
    OrganizationUser::create(organization.id, user.id, vec![Roles::OrgMember])
        .commit(project.get_connection())
        .unwrap();
    let user_results = organization.users(project.get_connection()).unwrap();
    assert_eq!(user_results.len(), 1);
    assert_eq!(user.id, user_results[0].1.id);
    let user_results2 = organization2.users(project.get_connection()).unwrap();
    assert_eq!(user_results2.len(), 2);
    assert_eq!(user2.id, user_results2[0].1.id);
    assert_eq!(user3.id, user_results2[1].1.id);

    // Add a new user to the organization
    OrganizationUser::create(organization.id, user2.id, vec![Roles::OrgMember])
        .commit(project.get_connection())
        .unwrap();
    let user_results = organization.users(project.get_connection()).unwrap();
    assert_eq!(user_results.len(), 2);
    assert_eq!(user.id, user_results[0].1.id);
    assert_eq!(user2.id, user_results[1].1.id);
    let user_results2 = organization2.users(project.get_connection()).unwrap();
    assert_eq!(user_results2.len(), 2);
    assert_eq!(user2.id, user_results2[0].1.id);
    assert_eq!(user3.id, user_results2[1].1.id);
}

#[test]
fn is_member() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let user3 = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_owner(&user)
        .with_user(&user2)
        .finish();
    // Reload for owner role
    let user = User::find(user.id.clone(), connection).unwrap();

    assert!(organization.is_member(&user, connection).unwrap());
    assert!(organization.is_member(&user2, connection).unwrap());
    assert!(!organization.is_member(&user3, connection).unwrap());
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

    let orgs = Organization::all_linked_to_user(user.id, project.get_connection()).unwrap();
    let mut test_vec = vec![org1, org2];
    test_vec.sort_by_key(|org| org.name.clone());
    assert_eq!(orgs, test_vec);
}

#[test]
fn all_org_names_linked_to_user() {
    let project = TestProject::new();
    let user1 = project.create_user().finish(); //Member and owner link
    let user2 = project.create_user().finish(); //Only owner link
    let user3 = project.create_user().finish(); //Only membership link
    let user4 = project.create_user().finish(); //No links
    let org1 = project
        .create_organization()
        .with_name(String::from("Test Org1"))
        .with_owner(&user1)
        .with_user(&user3)
        .finish();
    let org2 = project
        .create_organization()
        .with_name(String::from("Test Org2"))
        .with_owner(&user2)
        .with_user(&user1)
        .finish();
    let user1_links =
        Organization::all_org_names_linked_to_user(user1.id, project.get_connection()).unwrap();
    let user2_links =
        Organization::all_org_names_linked_to_user(user2.id, project.get_connection()).unwrap();
    let user3_links =
        Organization::all_org_names_linked_to_user(user3.id, project.get_connection()).unwrap();
    let user4_links =
        Organization::all_org_names_linked_to_user(user4.id, project.get_connection()).unwrap();
    let role_owner_string = [String::from("OrgOwner")];
    let role_member_string = [String::from("OrgMember")];
    //User1 has 2 links, owner of Org1 and member of Org2
    assert_eq!(user1_links.len(), 2);
    assert_eq!(user1_links[0].id, org1.id);
    assert_eq!(user1_links[1].id, org2.id);

    assert_eq!(user1_links[0].role, role_owner_string);
    assert_eq!(user1_links[1].role, role_member_string);

    //User2 has only 1 owner link with Org2
    assert_eq!(user2_links.len(), 1);
    assert_eq!(user2_links[0].id, org2.id);
    assert_eq!(user2_links[0].role, role_owner_string);
    //User3 has only 1 member link with Org1
    assert_eq!(user3_links.len(), 1);
    assert_eq!(user3_links[0].id, org1.id);
    assert_eq!(user3_links[0].role, role_member_string);
    //User4 has no links
    assert_eq!(user4_links.len(), 0);
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

    let orgs = Organization::all(project.get_connection()).unwrap();
    let mut test_vec = vec![org1, org2, org3];
    test_vec.sort_by_key(|org| org.name.clone());
    assert_eq!(orgs, test_vec);
}

#[test]
fn remove_users() {
    let project = TestProject::new();
    let user = project.create_user().with_last_name("user1").finish();
    let user2 = project.create_user().with_last_name("user2").finish();
    let user3 = project.create_user().with_last_name("user3").finish();
    let organization = project.create_organization().with_owner(&user).finish();
    OrganizationUser::create(organization.id, user2.id, vec![Roles::OrgMember])
        .commit(project.get_connection())
        .unwrap();
    OrganizationUser::create(organization.id, user3.id, vec![Roles::OrgMember])
        .commit(project.get_connection())
        .unwrap();
    let user2_id = user2.id;

    let mut user_results = organization.users(project.get_connection()).unwrap();

    user_results.sort_by_key(|k| k.1.last_name.clone());
    let mut users_before_delete = vec![user.clone(), user2, user3.clone()];
    users_before_delete.sort_by_key(|k| k.last_name.clone());

    assert_eq!(user_results[0].1, users_before_delete[0]);
    assert_eq!(user_results[1].1, users_before_delete[1]);
    assert_eq!(user_results[2].1, users_before_delete[2]);
    assert_eq!(user_results.len(), 3);

    //remove user
    let result = organization
        .remove_user(user2_id, project.get_connection())
        .unwrap();
    assert_eq!(result, 1);
    let user_results2: Vec<User> = organization
        .users(project.get_connection())
        .unwrap()
        .into_iter()
        .map(|u| u.1)
        .collect();
    let users_post_delete = vec![user, user3];

    assert_eq!(user_results2, users_post_delete);
}

#[test]
pub fn get_roles_for_user() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let mut user3 = project.create_user().finish();
    let user4 = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_owner(&user)
        .with_user(&user2)
        .finish();
    user3 = user3.add_role(Roles::Admin, connection).unwrap();

    assert_eq!(
        organization.get_roles_for_user(&user, connection).unwrap(),
        vec![Roles::OrgOwner]
    );
    assert_eq!(
        organization.get_roles_for_user(&user2, connection).unwrap(),
        vec![Roles::OrgMember]
    );
    assert_eq!(
        organization.get_roles_for_user(&user3, connection).unwrap(),
        vec![Roles::OrgOwner]
    );
    assert!(organization
        .get_roles_for_user(&user4, connection)
        .unwrap()
        .is_empty());
}

#[test]
pub fn get_scopes_for_user() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let mut user3 = project.create_user().finish();
    let user4 = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_owner(&user)
        .with_user(&user2)
        .finish();
    user3 = user3.add_role(Roles::Admin, connection).unwrap();

    assert_eq!(
        organization.get_scopes_for_user(&user, connection).unwrap(),
        vec![
            "artist:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "event:interest",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "order::make-external-payment",
            "order:read",
            "org:admin-users",
            "org:fans",
            "org:read",
            "org:users",
            "org:write",
            "redeem:ticket",
            "ticket:admin",
            "ticket:transfer",
            "user:read",
            "venue:write"
        ]
    );
    assert_eq!(
        organization
            .get_scopes_for_user(&user2, connection)
            .unwrap(),
        vec![
            "artist:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "event:interest",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "order:read",
            "org:fans",
            "org:read",
            "redeem:ticket",
            "ticket:admin",
            "ticket:transfer",
            "venue:write",
        ]
    );
    assert_eq!(
        organization
            .get_scopes_for_user(&user3, connection)
            .unwrap(),
        vec![
            "artist:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "event:interest",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "order::make-external-payment",
            "order:read",
            "org:admin-users",
            "org:fans",
            "org:read",
            "org:users",
            "org:write",
            "redeem:ticket",
            "ticket:admin",
            "ticket:transfer",
            "user:read",
            "venue:write"
        ]
    );
    assert!(organization
        .get_scopes_for_user(&user4, connection)
        .unwrap()
        .is_empty());
}

#[test]
fn add_user() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let organization = project.create_organization().with_owner(&user).finish();
    let organization_user = organization
        .add_user(user2.id, vec![Roles::OrgMember], connection)
        .unwrap();

    assert_eq!(organization_user.user_id, user2.id);
    assert_eq!(organization_user.organization_id, organization.id);
    assert_eq!(organization_user.id.to_string().is_empty(), false);
    let user2 = User::find(user2.id, connection).unwrap();
    assert!(organization
        .get_roles_for_user(&user2, connection)
        .unwrap()
        .contains(&Roles::OrgMember));
}

#[test]
fn add_fee_schedule() {
    let project = TestProject::new();
    let organization = project.create_organization().finish();
    let fee_structure = project.create_fee_schedule().finish();
    organization
        .add_fee_schedule(&fee_structure, project.get_connection())
        .unwrap();
    let organization = Organization::find(organization.id, project.get_connection()).unwrap();
    assert_eq!(organization.fee_schedule_id, fee_structure.id);
}

#[test]
fn search_fans() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let organization = project.create_organization().finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    project
        .create_order()
        .for_event(&event)
        .for_user(&user)
        .finish();

    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    cart.update_quantities(
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 5,
            redemption_code: None,
        }],
        false,
        connection,
    )
    .unwrap();
    let total = cart.calculate_total(connection).unwrap();

    cart.add_external_payment(Some("test".to_string()), user.id, total, connection)
        .unwrap();

    let search_results = organization
        .search_fans(
            None,
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    assert_eq!(search_results.data[0].user_id, user.id);
    let search_results = organization
        .search_fans(
            Some("NOT A REAL NAME".to_string()),
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    assert_eq!(search_results.data.len(), 0);
}
