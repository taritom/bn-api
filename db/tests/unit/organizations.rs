use bigneon_db::dev::TestProject;
use bigneon_db::prelude::*;
use bigneon_db::schema::*;
use bigneon_db::utils::dates;
use diesel;
use diesel::prelude::*;
use uuid::Uuid;

#[test]
fn create() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let fee_schedule = FeeSchedule::create(
        Uuid::nil(),
        format!("Zero fees",).into(),
        vec![NewFeeScheduleRange {
            min_price_in_cents: 0,
            company_fee_in_cents: 0,
            client_fee_in_cents: 0,
        }],
    )
    .commit(None, connection)
    .unwrap();

    let updated_fee_schedule = FeeSchedule::find(fee_schedule.id, connection).unwrap();
    assert_eq!(updated_fee_schedule.organization_id, Uuid::nil());

    let mut organization = Organization::create("Organization", fee_schedule.id);
    organization.sendgrid_api_key = Some("A_Test_Key".to_string());

    let mut organization = organization
        .commit(&"encryption_key".to_string(), None, connection)
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

    let updated_fee_schedule = FeeSchedule::find(fee_schedule.id, connection).unwrap();

    assert_eq!(updated_fee_schedule.organization_id, organization.id);
}

#[test]
fn find_by_asset_id() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project
        .create_event()
        .with_ticket_pricing()
        .with_ticket_type_count(2)
        .finish();
    let event2 = project
        .create_event()
        .with_ticket_pricing()
        .with_ticket_type_count(1)
        .finish();
    let ticket_type = &event.ticket_types(true, None, &connection).unwrap()[0];
    let ticket_type2 = &event2.ticket_types(true, None, &connection).unwrap()[0];
    let asset = Asset::find_by_ticket_type(ticket_type.id, connection).unwrap();
    let asset2 = Asset::find_by_ticket_type(ticket_type2.id, connection).unwrap();
    assert_eq!(
        Organization::find_by_asset_id(asset.id, connection).unwrap(),
        event.organization(connection).unwrap()
    );
    assert_eq!(
        Organization::find_by_asset_id(asset2.id, connection).unwrap(),
        event2.organization(connection).unwrap()
    );
}

#[test]
fn find_by_ticket_type_ids() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project
        .create_event()
        .with_ticket_pricing()
        .with_ticket_type_count(2)
        .finish();
    let event2 = project
        .create_event()
        .with_ticket_pricing()
        .with_ticket_type_count(1)
        .finish();
    let ticket_types = event.ticket_types(true, None, &connection).unwrap();
    let ticket_type = &ticket_types[0];
    let ticket_type2 = &ticket_types[1];
    let ticket_types = event2.ticket_types(true, None, &connection).unwrap();
    let ticket_type3 = &ticket_types[0];

    let organizations = Organization::find_by_ticket_type_ids(
        vec![ticket_type.id, ticket_type2.id, ticket_type3.id],
        connection,
    )
    .unwrap();
    assert_eq!(2, organizations.len());
    assert!(organizations.contains(&event.organization(connection).unwrap()));
    assert!(organizations.contains(&event2.organization(connection).unwrap()));
}

#[test]
fn find_by_order_item_ids() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project
        .create_organization()
        .with_name("Organization1".into())
        .with_fees()
        .finish();
    let organization2 = project
        .create_organization()
        .with_name("Organization2".into())
        .with_fees()
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let event2 = project
        .create_event()
        .with_organization(&organization2)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user = project.create_user().finish();
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    let user2 = project.create_user().finish();
    let mut cart2 = Order::find_or_create_cart(&user2, connection).unwrap();
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];
    let ticket_type2 = &event2.ticket_types(true, None, connection).unwrap()[0];
    cart.update_quantities(
        user.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: None,
        }],
        false,
        false,
        connection,
    )
    .unwrap();
    cart2
        .update_quantities(
            user2.id,
            &[UpdateOrderItem {
                ticket_type_id: ticket_type2.id,
                quantity: 10,
                redemption_code: None,
            }],
            false,
            false,
            connection,
        )
        .unwrap();

    let items = cart.items(&connection).unwrap();
    let items2 = cart2.items(&connection).unwrap();
    let order_item = items
        .iter()
        .find(|i| i.ticket_type_id == Some(ticket_type.id))
        .unwrap();
    let order_item2 = items2
        .iter()
        .find(|i| i.ticket_type_id == Some(ticket_type2.id))
        .unwrap();

    // Ticket belonging to only first event / organization
    let organizations =
        Organization::find_by_order_item_ids(&vec![order_item.id], connection).unwrap();
    assert_eq!(organizations, vec![organization.clone()]);

    // Ticket belonging to only second event / organization
    let organizations =
        Organization::find_by_order_item_ids(&vec![order_item2.id], connection).unwrap();
    assert_eq!(organizations, vec![organization2.clone()]);

    // Ticket belonging to both events / organizations
    let organizations =
        Organization::find_by_order_item_ids(&vec![order_item.id, order_item2.id], connection)
            .unwrap();
    assert_eq!(organizations, vec![organization, organization2]);
}

#[test]
fn has_fan() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let organization = project.create_organization().with_fees().finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];

    // No relationship
    assert!(!organization.has_fan(&user, connection).unwrap());

    // User adds item to cart but does not checkout so no relationship
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    cart.update_quantities(
        user.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: None,
        }],
        false,
        false,
        connection,
    )
    .unwrap();
    assert!(!organization.has_fan(&user, connection).unwrap());

    // User checks out so has a paid order so relationship exists
    assert_eq!(cart.calculate_total(connection).unwrap(), 1700);
    cart.add_external_payment(
        Some("test".to_string()),
        ExternalPaymentType::CreditCard,
        user.id,
        1700,
        connection,
    )
    .unwrap();
    assert_eq!(cart.status, OrderStatus::Paid);
    assert!(organization.has_fan(&user, connection).unwrap());
}

#[test]
fn update() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    //Edit Organization
    let mut edited_organization = project
        .create_organization()
        .with_member(&user, Roles::OrgOwner)
        .finish();

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
        .with_member(&user, Roles::OrgOwner)
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
        .with_member(&user, Roles::OrgOwner)
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
    let organization = project
        .create_organization()
        .with_member(&user, Roles::OrgOwner)
        .finish();
    let organization2 = project
        .create_organization()
        .with_member(&user3, Roles::OrgOwner)
        .finish();
    OrganizationUser::create(
        organization2.id,
        user2.id,
        vec![Roles::OrgMember],
        Vec::new(),
    )
    .commit(project.get_connection())
    .unwrap();

    // Owner is included in the user results for organization2 but not organization2
    let user_results = organization.users(None, project.get_connection()).unwrap();
    assert_eq!(user_results.len(), 1);
    assert_eq!(user.id, user_results[0].1.id);

    let user_results = organization2.users(None, project.get_connection()).unwrap();
    assert_eq!(
        vec![user2.id, user3.id],
        user_results.iter().map(|u| u.1.id).collect::<Vec<Uuid>>()
    );

    // Explicitly make the organization user an org user
    OrganizationUser::create(organization.id, user.id, vec![Roles::OrgMember], Vec::new())
        .commit(project.get_connection())
        .unwrap();
    let user_results = organization.users(None, project.get_connection()).unwrap();
    assert_eq!(user_results.len(), 1);
    assert_eq!(user.id, user_results[0].1.id);
    let user_results2 = organization2.users(None, project.get_connection()).unwrap();
    assert_eq!(user_results2.len(), 2);
    assert_eq!(user2.id, user_results2[0].1.id);
    assert_eq!(user3.id, user_results2[1].1.id);

    // Add a new user to the organization
    OrganizationUser::create(
        organization.id,
        user2.id,
        vec![Roles::OrgMember],
        Vec::new(),
    )
    .commit(project.get_connection())
    .unwrap();
    let user_results = organization.users(None, project.get_connection()).unwrap();
    assert_eq!(user_results.len(), 2);
    assert_eq!(user.id, user_results[0].1.id);
    assert_eq!(user2.id, user_results[1].1.id);
    let user_results2 = organization2.users(None, project.get_connection()).unwrap();
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
        .with_member(&user, Roles::OrgOwner)
        .with_member(&user2, Roles::OrgMember)
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
    let org1 = project
        .create_organization()
        .with_member(&user, Roles::OrgOwner)
        .finish();
    let org2 = project
        .create_organization()
        .with_member(&user2, Roles::OrgOwner)
        .with_member(&user, Roles::OrgMember)
        .finish();
    let _org3 = project
        .create_organization()
        .with_member(&user2, Roles::OrgOwner)
        .finish();

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
        .with_member(&user1, Roles::OrgOwner)
        .with_member(&user3, Roles::OrgMember)
        .finish();
    let org2 = project
        .create_organization()
        .with_name(String::from("Test Org2"))
        .with_member(&user2, Roles::OrgOwner)
        .with_member(&user1, Roles::OrgMember)
        .finish();
    let user1_links =
        Organization::all_org_names_linked_to_user(user1.id, project.get_connection()).unwrap();
    let user2_links =
        Organization::all_org_names_linked_to_user(user2.id, project.get_connection()).unwrap();
    let user3_links =
        Organization::all_org_names_linked_to_user(user3.id, project.get_connection()).unwrap();
    let user4_links =
        Organization::all_org_names_linked_to_user(user4.id, project.get_connection()).unwrap();

    //User1 has 2 links, owner of Org1 and member of Org2
    assert_eq!(user1_links.len(), 2);
    assert_eq!(user1_links[0].id, org1.id);
    assert_eq!(user1_links[1].id, org2.id);

    assert_eq!(user1_links[0].role, vec![Roles::OrgOwner]);
    assert_eq!(user1_links[1].role, vec![Roles::OrgMember]);

    //User2 has only 1 owner link with Org2
    assert_eq!(user2_links.len(), 1);
    assert_eq!(user2_links[0].id, org2.id);
    assert_eq!(user2_links[0].role, vec![Roles::OrgOwner]);
    //User3 has only 1 member link with Org1
    assert_eq!(user3_links.len(), 1);
    assert_eq!(user3_links[0].id, org1.id);
    assert_eq!(user3_links[0].role, vec![Roles::OrgMember]);
    //User4 has no links
    assert_eq!(user4_links.len(), 0);
}

#[test]
fn all() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let org1 = project
        .create_organization()
        .with_member(&user, Roles::OrgOwner)
        .finish();
    let org2 = project
        .create_organization()
        .with_member(&user2, Roles::OrgOwner)
        .with_member(&user, Roles::OrgMember)
        .finish();
    let org3 = project
        .create_organization()
        .with_member(&user2, Roles::OrgOwner)
        .finish();

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
    let organization = project
        .create_organization()
        .with_member(&user, Roles::OrgOwner)
        .finish();
    OrganizationUser::create(
        organization.id,
        user2.id,
        vec![Roles::OrgMember],
        Vec::new(),
    )
    .commit(project.get_connection())
    .unwrap();
    OrganizationUser::create(
        organization.id,
        user3.id,
        vec![Roles::OrgMember],
        Vec::new(),
    )
    .commit(project.get_connection())
    .unwrap();
    let user2_id = user2.id;

    let mut user_results = organization.users(None, project.get_connection()).unwrap();

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
        .users(None, project.get_connection())
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
        .with_member(&user, Roles::OrgOwner)
        .with_member(&user2, Roles::OrgMember)
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
    let owner = project.create_user().finish();
    let org_admin = project.create_user().finish();
    let box_office = project.create_user().finish();
    let door_person = project.create_user().finish();
    let member = project.create_user().finish();
    let no_access_user = project.create_user().finish();
    let promoter = project.create_user().finish();
    let promoter_read_only = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_member(&owner, Roles::OrgOwner)
        .with_member(&member, Roles::OrgMember)
        .with_member(&org_admin, Roles::OrgAdmin)
        .with_member(&box_office, Roles::OrgBoxOffice)
        .with_member(&door_person, Roles::DoorPerson)
        .with_member(&promoter, Roles::Promoter)
        .with_member(&promoter_read_only, Roles::PromoterReadOnly)
        .finish();
    let mut admin = project.create_user().finish();
    admin = admin.add_role(Roles::Admin, connection).unwrap();

    assert_equiv!(
        organization
            .get_scopes_for_user(&owner, connection)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
        vec![
            "artist:write",
            "box-office-ticket:read",
            "box-office-ticket:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "dashboard:read",
            "event:broadcast",
            "event:cancel",
            "event:financial-reports",
            "event:interest",
            "event:reports",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "note:delete",
            "note:read",
            "note:write",
            "order:make-external-payment",
            "order:read",
            "order:read-own",
            "order:refund",
            "org:admin-users",
            "org:fans",
            "org:read",
            "org:read-events",
            "org:reports",
            "org:users",
            "org:write",
            "redeem:ticket",
            "ticket:admin",
            "ticket:read",
            "ticket:transfer",
            "ticket:write",
            "ticket:write-own",
            "ticket-type:read",
            "ticket-type:write",
            "transfer:cancel",
            "transfer:cancel-own",
            "transfer:read",
            "transfer:read-own",
            "user:read",
            "venue:write"
        ]
    );

    assert_equiv!(
        organization
            .get_scopes_for_user(&org_admin, connection)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
        vec![
            "artist:write",
            "box-office-ticket:read",
            "box-office-ticket:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "dashboard:read",
            "event:broadcast",
            "event:cancel",
            "event:financial-reports",
            "event:interest",
            "event:reports",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "note:delete",
            "note:read",
            "note:write",
            "order:make-external-payment",
            "order:read",
            "order:read-own",
            "order:refund",
            "org:fans",
            "org:read",
            "org:read-events",
            "org:reports",
            "org:users",
            "org:write",
            "redeem:ticket",
            "ticket:admin",
            "ticket:read",
            "ticket:transfer",
            "ticket:write",
            "ticket:write-own",
            "ticket-type:read",
            "ticket-type:write",
            "transfer:cancel",
            "transfer:cancel-own",
            "transfer:read",
            "transfer:read-own",
            "user:read",
            "venue:write"
        ]
    );

    assert_equiv!(
        organization
            .get_scopes_for_user(&box_office, connection)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
        vec![
            "box-office-ticket:read",
            "code:read",
            "event:scan",
            "event:view-guests",
            "hold:read",
            "note:read",
            "note:write",
            "order:make-external-payment",
            "order:read",
            "org:read-events",
            "redeem:ticket",
            "ticket:read",
            "dashboard:read",
        ]
    );

    assert_equiv!(
        organization
            .get_scopes_for_user(&door_person, connection)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
        vec![
            "code:read",
            "event:scan",
            "hold:read",
            "note:read",
            "note:write",
            "order:read",
            "org:read-events",
            "redeem:ticket",
            "ticket:read",
            "event:view-guests",
            "dashboard:read",
        ]
    );

    assert_equiv!(
        organization
            .get_scopes_for_user(&promoter, connection)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
        vec![
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "dashboard:read",
            "event:interest",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "note:read",
            "note:write",
            "order:read",
            "org:read-events",
            "ticket:read",
            "ticket-type:read",
            "ticket-type:write",
            "transfer:cancel",
            "transfer:read"
        ]
    );

    assert_equiv!(
        organization
            .get_scopes_for_user(&promoter_read_only, connection)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
        vec![
            "code:read",
            "comp:read",
            "dashboard:read",
            "event:interest",
            "event:view-guests",
            "hold:read",
            "note:read",
            "order:read",
            "org:read-events",
            "ticket:read",
            "ticket-type:read",
            "transfer:read"
        ]
    );

    assert_equiv!(
        organization
            .get_scopes_for_user(&member, connection)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
        vec![
            "artist:write",
            "box-office-ticket:read",
            "box-office-ticket:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "dashboard:read",
            "event:cancel",
            "event:interest",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "note:read",
            "note:write",
            "order:read",
            "order:read-own",
            "order:refund",
            "org:fans",
            "org:read",
            "org:read-events",
            "redeem:ticket",
            "ticket:admin",
            "ticket:read",
            "ticket:transfer",
            "ticket:write-own",
            "ticket-type:read",
            "ticket-type:write",
            "transfer:cancel",
            "transfer:cancel-own",
            "transfer:read",
            "transfer:read-own",
            "venue:write",
        ]
    );

    assert_equiv!(
        organization
            .get_scopes_for_user(&admin, connection)
            .unwrap()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>(),
        vec![
            "artist:write",
            "box-office-ticket:read",
            "box-office-ticket:write",
            "code:read",
            "code:write",
            "comp:read",
            "comp:write",
            "dashboard:read",
            "event:broadcast",
            "event:cancel",
            "event:financial-reports",
            "event:interest",
            "event:reports",
            "event:scan",
            "event:view-guests",
            "event:write",
            "hold:read",
            "hold:write",
            "note:delete",
            "note:read",
            "note:write",
            "order:make-external-payment",
            "order:read",
            "order:read-own",
            "order:refund",
            "org:admin-users",
            "org:fans",
            "org:read",
            "org:read-events",
            "org:reports",
            "org:users",
            "org:write",
            "redeem:ticket",
            "ticket:admin",
            "ticket:read",
            "ticket:transfer",
            "ticket:write",
            "ticket:write-own",
            "ticket-type:read",
            "ticket-type:write",
            "transfer:cancel",
            "transfer:cancel-own",
            "transfer:read",
            "transfer:read-own",
            "user:read",
            "venue:write"
        ]
    );

    assert!(organization
        .get_scopes_for_user(&no_access_user, connection)
        .unwrap()
        .is_empty());
}

#[test]
fn add_user() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_member(&user, Roles::OrgOwner)
        .finish();
    let organization_user = organization
        .add_user(user2.id, vec![Roles::OrgMember], Vec::new(), connection)
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
    let fee_structure = project.create_fee_schedule().finish(None);

    let updated_fee_schedule =
        FeeSchedule::find(fee_structure.id, project.get_connection()).unwrap();
    assert_eq!(updated_fee_schedule.organization_id, Uuid::nil());

    organization
        .add_fee_schedule(&fee_structure, project.get_connection())
        .unwrap();
    let organization = Organization::find(organization.id, project.get_connection()).unwrap();
    assert_eq!(organization.fee_schedule_id, fee_structure.id);

    let updated_fee_schedule =
        FeeSchedule::find(fee_structure.id, project.get_connection()).unwrap();
    assert_eq!(updated_fee_schedule.organization_id, organization.id);
}

#[test]
fn search_fans() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let organization = project.create_organization().finish();
    // Has a refunded order and later a second order and transferred ticket from order_user3
    let order_user = project.create_user().finish();
    // Has a normal order and a box office order on their behalf and has an event interest
    let order_user2 = project.create_user().finish();
    // Has a box office order on their behalf
    let order_user3 = project.create_user().finish();
    // Has only box office orders for it / not in the results as a fan
    let box_office_user = project.create_user().finish();
    // Has previously been transferred to and transferred their tickets on
    let previous_transfer_user = project.create_user().finish();
    // Has a transferred ticket which makes them a fan
    let transfer_user = project.create_user().finish();
    // Just event interested
    let interested_user = project.create_user().finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let event_interest = project
        .create_event_interest()
        .with_user(&order_user2)
        .with_event(&event)
        .finish();
    let event_interest: EventInterest =
        diesel::update(event_interest::table.filter(event_interest::id.eq(event_interest.id)))
            .set((event_interest::created_at.eq(dates::now().add_hours(-12).finish()),))
            .get_result(connection)
            .unwrap();
    let event_interest2 = project
        .create_event_interest()
        .with_user(&interested_user)
        .with_event(&event)
        .finish();
    let event_interest2: EventInterest =
        diesel::update(event_interest::table.filter(event_interest::id.eq(event_interest2.id)))
            .set((event_interest::created_at.eq(dates::now().add_hours(-11).finish()),))
            .get_result(connection)
            .unwrap();

    let order = project
        .create_order()
        .for_user(&order_user)
        .for_event(&event)
        .quantity(5)
        .is_paid()
        .finish();
    let mut order: Order = diesel::update(orders::table.filter(orders::id.eq(order.id)))
        .set((orders::order_date.eq(dates::now().add_hours(-10).finish()),))
        .get_result(connection)
        .unwrap();
    let order2 = project
        .create_order()
        .for_user(&order_user2)
        .for_event(&event)
        .quantity(5)
        .is_paid()
        .finish();
    let order2: Order = diesel::update(orders::table.filter(orders::id.eq(order2.id)))
        .set((orders::order_date.eq(dates::now().add_hours(-9).finish()),))
        .get_result(connection)
        .unwrap();
    let order3 = project
        .create_order()
        .for_user(&box_office_user)
        .on_behalf_of_user(&order_user2)
        .for_event(&event)
        .quantity(5)
        .is_paid()
        .finish();
    let order3: Order = diesel::update(orders::table.filter(orders::id.eq(order3.id)))
        .set((orders::order_date.eq(dates::now().add_hours(-8).finish()),))
        .get_result(connection)
        .unwrap();
    let order4 = project
        .create_order()
        .for_user(&box_office_user)
        .on_behalf_of_user(&order_user3)
        .for_event(&event)
        .quantity(5)
        .is_paid()
        .finish();
    let order4: Order = diesel::update(orders::table.filter(orders::id.eq(order4.id)))
        .set((orders::order_date.eq(dates::now().add_hours(-7).finish()),))
        .get_result(connection)
        .unwrap();

    // Expected results after initial orders
    let mut expected_results = vec![
        order_user.id,
        order_user2.id,
        order_user3.id,
        interested_user.id,
    ];
    expected_results.sort();
    let search_results = organization
        .search_fans(
            None,
            None,
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    let mut results: Vec<Uuid> = search_results.data.iter().map(|f| f.user_id).collect();
    results.sort();
    assert_eq!(results, expected_results);
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user.id,
            first_name: order_user.first_name.clone(),
            last_name: order_user.last_name.clone(),
            email: order_user.email.clone(),
            phone: order_user.phone.clone(),
            thumb_profile_pic_url: order_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(1),
            created_at: order_user.created_at,
            first_order_time: Some(order.order_date),
            last_order_time: Some(order.order_date),
            revenue_in_cents: Some(order.calculate_total(connection).unwrap()),
            first_interaction_time: Some(order.order_date),
            last_interaction_time: Some(order.order_date),
        }
    );
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user2.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user2.id,
            first_name: order_user2.first_name.clone(),
            last_name: order_user2.last_name.clone(),
            email: order_user2.email.clone(),
            phone: order_user2.phone.clone(),
            thumb_profile_pic_url: order_user2.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(2),
            created_at: order_user2.created_at,
            first_order_time: Some(order2.order_date),
            last_order_time: Some(order3.order_date),
            revenue_in_cents: Some(
                order2.calculate_total(connection).unwrap()
                    + order3.calculate_total(connection).unwrap()
            ),
            first_interaction_time: Some(event_interest.created_at),
            last_interaction_time: Some(order3.order_date),
        }
    );
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user3.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user3.id,
            first_name: order_user3.first_name.clone(),
            last_name: order_user3.last_name.clone(),
            email: order_user3.email.clone(),
            phone: order_user3.phone.clone(),
            thumb_profile_pic_url: order_user3.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(1),
            created_at: order_user3.created_at,
            first_order_time: Some(order4.order_date),
            last_order_time: Some(order4.order_date),
            revenue_in_cents: Some(order4.calculate_total(connection).unwrap()),
            first_interaction_time: Some(order4.order_date),
            last_interaction_time: Some(order4.order_date),
        }
    );
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == interested_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: interested_user.id,
            first_name: interested_user.first_name.clone(),
            last_name: interested_user.last_name.clone(),
            email: interested_user.email.clone(),
            phone: interested_user.phone.clone(),
            thumb_profile_pic_url: interested_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(0),
            created_at: interested_user.created_at,
            first_order_time: None,
            last_order_time: None,
            revenue_in_cents: Some(0),
            first_interaction_time: Some(event_interest2.created_at),
            last_interaction_time: Some(event_interest2.created_at),
        }
    );

    // Initial order is refunded -- it should still show the user but with new order details
    let items = order.items(&connection).unwrap();
    let order_item = items
        .iter()
        .find(|i| i.item_type == OrderItemTypes::Tickets)
        .unwrap();
    let tickets = TicketInstance::find_for_order_item(order_item.id, connection).unwrap();
    let refund_items: Vec<RefundItemRequest> = tickets
        .iter()
        .map(|t| RefundItemRequest {
            order_item_id: order_item.id,
            ticket_instance_id: Some(t.id),
        })
        .collect();
    let refund = order
        .refund(&refund_items, order_user.id, None, connection)
        .unwrap()
        .0;
    let refund: Refund = diesel::update(refunds::table.filter(refunds::id.eq(refund.id)))
        .set((refunds::created_at.eq(dates::now().add_hours(-6).finish()),))
        .get_result(connection)
        .unwrap();
    let mut expected_results = vec![
        order_user.id,
        order_user2.id,
        order_user3.id,
        interested_user.id,
    ];
    expected_results.sort();
    let search_results = organization
        .search_fans(
            None,
            None,
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    let mut results: Vec<Uuid> = search_results.data.iter().map(|f| f.user_id).collect();
    results.sort();
    assert_eq!(results, expected_results);
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user.id,
            first_name: order_user.first_name.clone(),
            last_name: order_user.last_name.clone(),
            email: order_user.email.clone(),
            phone: order_user.phone.clone(),
            thumb_profile_pic_url: order_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(1),
            created_at: order_user.created_at,
            first_order_time: Some(order.order_date),
            last_order_time: Some(order.order_date),
            revenue_in_cents: Some(0),
            first_interaction_time: Some(order.order_date),
            last_interaction_time: Some(refund.created_at),
        }
    );

    // Second order shows only non refunded price shows for user
    let order5 = project
        .create_order()
        .for_user(&order_user)
        .for_event(&event)
        .quantity(5)
        .is_paid()
        .finish();
    let order5: Order = diesel::update(orders::table.filter(orders::id.eq(order5.id)))
        .set((orders::order_date.eq(dates::now().add_hours(-5).finish()),))
        .get_result(connection)
        .unwrap();
    let mut expected_results = vec![
        order_user.id,
        order_user2.id,
        order_user3.id,
        interested_user.id,
    ];
    expected_results.sort();
    let search_results = organization
        .search_fans(
            None,
            None,
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    let mut results: Vec<Uuid> = search_results.data.iter().map(|f| f.user_id).collect();
    results.sort();
    assert_eq!(results, expected_results);
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user.id,
            first_name: order_user.first_name.clone(),
            last_name: order_user.last_name.clone(),
            email: order_user.email.clone(),
            phone: order_user.phone.clone(),
            thumb_profile_pic_url: order_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(2),
            created_at: order_user.created_at,
            first_order_time: Some(order.order_date),
            last_order_time: Some(order5.order_date),
            revenue_in_cents: Some(order5.calculate_total(connection).unwrap()),
            first_interaction_time: Some(order.order_date),
            last_interaction_time: Some(order5.order_date),
        }
    );

    // User transfers some tickets, most to a new user, 1 to existing user with other purchases all 4 should now show
    let mut ticket_ids: Vec<Uuid> = TicketInstance::find_for_user(order_user3.id, connection)
        .unwrap()
        .iter()
        .map(|t| t.id)
        .collect();
    let transfer = TicketInstance::direct_transfer(
        order_user3.id,
        &vec![ticket_ids.pop().unwrap()],
        "nowhere",
        TransferMessageType::Email,
        order_user.id,
        connection,
    )
    .unwrap();
    let transfer: Transfer = diesel::update(transfers::table.filter(transfers::id.eq(transfer.id)))
        .set((
            transfers::created_at.eq(dates::now().add_hours(-4).finish()),
            transfers::updated_at.eq(dates::now().add_hours(-4).finish()),
        ))
        .get_result(connection)
        .unwrap();
    // Intermediary transfer user transfers out their inventory
    let intermediary_transfer = TicketInstance::direct_transfer(
        order_user3.id,
        &ticket_ids,
        "nowhere",
        TransferMessageType::Email,
        previous_transfer_user.id,
        connection,
    )
    .unwrap();
    let intermediary_transfer: Transfer =
        diesel::update(transfers::table.filter(transfers::id.eq(intermediary_transfer.id)))
            .set((
                transfers::created_at.eq(dates::now().add_hours(-3).finish()),
                transfers::updated_at.eq(dates::now().add_hours(-3).finish()),
            ))
            .get_result(connection)
            .unwrap();
    let transfer2 = TicketInstance::direct_transfer(
        previous_transfer_user.id,
        &ticket_ids,
        "nowhere",
        TransferMessageType::Email,
        transfer_user.id,
        connection,
    )
    .unwrap();
    let transfer2: Transfer =
        diesel::update(transfers::table.filter(transfers::id.eq(transfer2.id)))
            .set((
                transfers::created_at.eq(dates::now().add_hours(-2).finish()),
                transfers::updated_at.eq(dates::now().add_hours(-2).finish()),
            ))
            .get_result(connection)
            .unwrap();
    let mut expected_results = vec![
        order_user.id,
        order_user2.id,
        order_user3.id,
        previous_transfer_user.id,
        transfer_user.id,
        interested_user.id,
    ];
    expected_results.sort();
    let search_results = organization
        .search_fans(
            None,
            None,
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    let mut results: Vec<Uuid> = search_results.data.iter().map(|f| f.user_id).collect();
    results.sort();
    assert_eq!(results, expected_results);
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user.id,
            first_name: order_user.first_name.clone(),
            last_name: order_user.last_name.clone(),
            email: order_user.email.clone(),
            phone: order_user.phone.clone(),
            thumb_profile_pic_url: order_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(2),
            created_at: order_user.created_at,
            first_order_time: Some(order.order_date),
            last_order_time: Some(order5.order_date),
            revenue_in_cents: Some(order5.calculate_total(connection).unwrap()),
            first_interaction_time: Some(order.order_date),
            last_interaction_time: Some(transfer.updated_at),
        }
    );
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user2.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user2.id,
            first_name: order_user2.first_name.clone(),
            last_name: order_user2.last_name.clone(),
            email: order_user2.email.clone(),
            phone: order_user2.phone.clone(),
            thumb_profile_pic_url: order_user2.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(2),
            created_at: order_user2.created_at,
            first_order_time: Some(order2.order_date),
            last_order_time: Some(order3.order_date),
            revenue_in_cents: Some(
                order2.calculate_total(connection).unwrap()
                    + order3.calculate_total(connection).unwrap()
            ),
            first_interaction_time: Some(event_interest.created_at),
            last_interaction_time: Some(order3.order_date),
        }
    );
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user3.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user3.id,
            first_name: order_user3.first_name.clone(),
            last_name: order_user3.last_name.clone(),
            email: order_user3.email.clone(),
            phone: order_user3.phone.clone(),
            thumb_profile_pic_url: order_user3.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(1),
            created_at: order_user3.created_at,
            first_order_time: Some(order4.order_date),
            last_order_time: Some(order4.order_date),
            revenue_in_cents: Some(order4.calculate_total(connection).unwrap()),
            first_interaction_time: Some(order4.order_date),
            last_interaction_time: Some(intermediary_transfer.updated_at),
        }
    );
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == previous_transfer_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: previous_transfer_user.id,
            first_name: previous_transfer_user.first_name.clone(),
            last_name: previous_transfer_user.last_name.clone(),
            email: previous_transfer_user.email.clone(),
            phone: previous_transfer_user.phone.clone(),
            thumb_profile_pic_url: previous_transfer_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(0),
            created_at: previous_transfer_user.created_at,
            first_order_time: None,
            last_order_time: None,
            revenue_in_cents: Some(0),
            first_interaction_time: Some(intermediary_transfer.created_at),
            last_interaction_time: Some(transfer2.updated_at),
        }
    );
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == transfer_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: transfer_user.id,
            first_name: transfer_user.first_name.clone(),
            last_name: transfer_user.last_name.clone(),
            email: transfer_user.email.clone(),
            phone: transfer_user.phone.clone(),
            thumb_profile_pic_url: transfer_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(0),
            created_at: transfer_user.created_at,
            first_order_time: None,
            last_order_time: None,
            revenue_in_cents: Some(0),
            first_interaction_time: Some(transfer2.created_at),
            last_interaction_time: Some(transfer2.updated_at),
        }
    );
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == interested_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: interested_user.id,
            first_name: interested_user.first_name.clone(),
            last_name: interested_user.last_name.clone(),
            email: interested_user.email.clone(),
            phone: interested_user.phone.clone(),
            thumb_profile_pic_url: interested_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(0),
            created_at: interested_user.created_at,
            first_order_time: None,
            last_order_time: None,
            revenue_in_cents: Some(0),
            first_interaction_time: Some(event_interest2.created_at),
            last_interaction_time: Some(event_interest2.created_at),
        }
    );

    // Redeem ticket causing user last_interaction_time to change
    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];
    let ticket = &order5.tickets(ticket_type.id, connection).unwrap()[0];
    TicketInstance::redeem_ticket(
        ticket.id,
        ticket.redeem_key.clone().unwrap(),
        order_user.id,
        connection,
    )
    .unwrap();
    let ticket: TicketInstance =
        diesel::update(ticket_instances::table.filter(ticket_instances::id.eq(ticket.id)))
            .set((ticket_instances::redeemed_at.eq(dates::now().add_hours(-1).finish()),))
            .get_result(connection)
            .unwrap();
    let search_results = organization
        .search_fans(
            None,
            None,
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user.id,
            first_name: order_user.first_name.clone(),
            last_name: order_user.last_name.clone(),
            email: order_user.email.clone(),
            phone: order_user.phone.clone(),
            thumb_profile_pic_url: order_user.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(2),
            created_at: order_user.created_at,
            first_order_time: Some(order.order_date),
            last_order_time: Some(order5.order_date),
            revenue_in_cents: Some(order5.calculate_total(connection).unwrap()),
            first_interaction_time: Some(order.order_date),
            last_interaction_time: ticket.redeemed_at,
        }
    );

    // Update event interest to be greater than orders (showing max is used)
    let event_interest: EventInterest =
        diesel::update(event_interest::table.filter(event_interest::id.eq(event_interest.id)))
            .set((event_interest::created_at.eq(dates::now().finish()),))
            .get_result(connection)
            .unwrap();
    let search_results = organization
        .search_fans(
            None,
            None,
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    assert_eq!(
        search_results
            .data
            .iter()
            .find(|f| f.user_id == order_user2.id)
            .unwrap(),
        &DisplayFan {
            user_id: order_user2.id,
            first_name: order_user2.first_name.clone(),
            last_name: order_user2.last_name.clone(),
            email: order_user2.email.clone(),
            phone: order_user2.phone.clone(),
            thumb_profile_pic_url: order_user2.thumb_profile_pic_url.clone(),
            organization_id: organization.id,
            order_count: Some(2),
            created_at: order_user2.created_at,
            first_order_time: Some(order2.order_date),
            last_order_time: Some(order3.order_date),
            revenue_in_cents: Some(
                order2.calculate_total(connection).unwrap()
                    + order3.calculate_total(connection).unwrap()
            ),
            first_interaction_time: Some(order2.order_date),
            last_interaction_time: Some(event_interest.created_at),
        }
    );

    // Filtering finding a given user
    let search_results = organization
        .search_fans(
            None,
            order_user.email.clone(),
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            &project.connection,
        )
        .unwrap();
    assert_eq!(search_results.data.len(), 1);
    assert_eq!(search_results.data[0].user_id, order_user.id);

    // Filtering returning nothing
    let search_results = organization
        .search_fans(
            None,
            Some("NOT A REAL NAME".to_string()),
            0,
            100,
            FanSortField::FirstName,
            SortingDir::Asc,
            connection,
        )
        .unwrap();
    assert_eq!(search_results.data.len(), 0);
}

#[test]
fn credit_card_fees() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_cc_fee(0f32)
        .with_event_fee()
        .finish();

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

    let ticket_type = &event.ticket_types(true, None, connection).unwrap()[0];

    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    cart.update_quantities(
        user.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 1,
            redemption_code: None,
        }],
        false,
        false,
        connection,
    )
    .unwrap();

    for i in cart.items(connection).unwrap().iter() {
        assert_ne!(i.item_type, OrderItemTypes::CreditCardFees);
    }

    let pre_cc_fee_total = cart.calculate_total(connection).unwrap();

    let org_update = OrganizationEditableAttributes {
        cc_fee_percent: Some(5f32),
        ..Default::default()
    };

    organization
        .update(org_update, &"encryption_key".to_string(), connection)
        .unwrap();

    cart.update_quantities(
        user.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 1,
            redemption_code: None,
        }],
        false,
        false,
        connection,
    )
    .unwrap();

    let mut cc_fee_count = 0;
    for i in cart.items(connection).unwrap().iter() {
        if i.item_type == OrderItemTypes::CreditCardFees {
            cc_fee_count += 1;
        }
    }
    assert_eq!(cc_fee_count, 1);
    assert_eq!(
        cart.calculate_total(connection).unwrap(),
        pre_cc_fee_total + (pre_cc_fee_total as f32 * (5f32 / 100f32)).round() as i64
    );
}
