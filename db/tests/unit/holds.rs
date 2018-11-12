use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::utils::errors::ErrorCode::ValidationError;

#[test]
fn create() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    Hold::create(
        "test".to_string(),
        event.id,
        "IHAVEACODE".to_string(),
        Some(0),
        None,
        Some(4),
        HoldTypes::Discount,
        event.ticket_types(db.get_connection()).unwrap()[0].id,
    ).commit(db.get_connection())
    .unwrap();
}

#[test]
fn create_with_validation_errors() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    let result = Hold::create(
        "test".to_string(),
        event.id,
        "IHAVEACODE".to_string(),
        None,
        None,
        Some(4),
        HoldTypes::Discount,
        event.ticket_types(db.get_connection()).unwrap()[0].id,
    ).commit(db.get_connection());

    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("discount_in_cents"));
                assert_eq!(errors["discount_in_cents"].len(), 1);
                assert_eq!(errors["discount_in_cents"][0].code, "required");
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Dupe redemption code
    let hold = db.create_hold().finish();
    let result = Hold::create(
        "test".to_string(),
        event.id,
        hold.redemption_code,
        Some(0),
        None,
        Some(4),
        HoldTypes::Discount,
        event.ticket_types(db.get_connection()).unwrap()[0].id,
    ).commit(db.get_connection());
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("redemption_code"));
                assert_eq!(errors["redemption_code"].len(), 1);
                assert_eq!(errors["redemption_code"][0].code, "uniqueness");
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Redemption code used by a code
    let code = db.create_code().finish();
    let result = Hold::create(
        "test".to_string(),
        event.id,
        code.redemption_code,
        Some(0),
        None,
        Some(4),
        HoldTypes::Discount,
        event.ticket_types(db.get_connection()).unwrap()[0].id,
    ).commit(db.get_connection());
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("redemption_code"));
                assert_eq!(errors["redemption_code"].len(), 1);
                assert_eq!(errors["redemption_code"][0].code, "uniqueness");
            }
            _ => panic!("Expected validation error"),
        },
    }
}

#[test]
fn update() {
    let db = TestProject::new();
    let hold = db.create_hold().finish();

    let update_patch = UpdateHoldAttributes {
        discount_in_cents: Some(Some(10)),
        max_per_order: Some(None),
        end_at: Some(None),
        name: Some("New name".to_string()),
        ..Default::default()
    };
    let new_hold = hold.update(update_patch, db.get_connection()).unwrap();
    assert_eq!(new_hold.name, "New name".to_string());
    assert_eq!(new_hold.max_per_order, None);
    assert_eq!(new_hold.end_at, None);
    assert_eq!(new_hold.discount_in_cents, Some(10));
}

#[test]
fn update_with_validation_errors() {
    let db = TestProject::new();
    let connection = db.get_connection();
    let hold = db.create_hold().with_hold_type(HoldTypes::Comp).finish();
    assert!(hold.discount_in_cents.is_none());

    let update_patch = UpdateHoldAttributes {
        hold_type: Some(HoldTypes::Discount.to_string()),
        ..Default::default()
    };
    let result = hold.update(update_patch, connection);
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("discount_in_cents"));
                assert_eq!(errors["discount_in_cents"].len(), 1);
                assert_eq!(errors["discount_in_cents"][0].code, "required");
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Dupe redemption code
    let hold2 = db.create_hold().finish();
    let update_patch = UpdateHoldAttributes {
        redemption_code: Some(hold2.redemption_code),
        ..Default::default()
    };
    let result = hold.update(update_patch, connection);
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("redemption_code"));
                assert_eq!(errors["redemption_code"].len(), 1);
                assert_eq!(errors["redemption_code"][0].code, "uniqueness");
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Dupe redemption code used by code
    let code = db.create_code().finish();
    let update_patch = UpdateHoldAttributes {
        redemption_code: Some(code.redemption_code),
        ..Default::default()
    };
    let result = hold.update(update_patch, connection);
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("redemption_code"));
                assert_eq!(errors["redemption_code"].len(), 1);
                assert_eq!(errors["redemption_code"][0].code, "uniqueness");
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Create and assign comp to active cart
    let comp = db.create_comp().with_hold(&hold).finish();
    let organization = db
        .create_organization()
        .with_fee_schedule(&db.create_fee_schedule().finish())
        .finish();
    let event = db
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user = db.create_user().finish();
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    cart.update_quantities(
        &vec![UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: Some(comp.redemption_code),
        }],
        connection,
    ).unwrap();

    // Try to change out of Comp type
    let update_patch = UpdateHoldAttributes {
        hold_type: Some(HoldTypes::Discount.to_string()),
        discount_in_cents: Some(Some(10)),
        ..Default::default()
    };
    let result = hold.update(update_patch, connection);
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("hold_type"));
                assert_eq!(errors["hold_type"].len(), 1);
                assert_eq!(errors["hold_type"][0].code, "comps_in_use");
            }
            _ => panic!("Expected validation error"),
        },
    }
}

#[test]
fn destroy() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let hold = project.create_hold().finish();
    assert!(hold.clone().destroy(connection).is_ok());
    assert!(Code::find(hold.id, connection).is_err());

    // Destroy hold with comps
    let comp = project.create_comp().finish();
    let hold = Hold::find(comp.hold_id, connection).unwrap();
    hold.clone().destroy(connection).unwrap();
    assert!(Code::find(hold.id, connection).is_err());
}

#[test]
fn destroy_with_validation_errors() {
    let db = TestProject::new();
    let connection = db.get_connection();
    // Create and assign comp to active cart
    let comp = db.create_comp().finish();
    let hold = Hold::find(comp.hold_id, connection).unwrap();
    let organization = db
        .create_organization()
        .with_fee_schedule(&db.create_fee_schedule().finish())
        .finish();
    let event = db
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let user = db.create_user().finish();
    let mut cart = Order::find_or_create_cart(&user, connection).unwrap();
    let ticket_type = &event.ticket_types(connection).unwrap()[0];
    cart.update_quantities(
        &vec![UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 1,
            redemption_code: Some(comp.redemption_code),
        }],
        connection,
    ).unwrap();

    // Try to destroy hold
    let result = hold.destroy(connection);
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("hold_type"));
                assert_eq!(errors["hold_type"].len(), 1);
                assert_eq!(errors["hold_type"][0].code, "comps_in_use");
            }
            _ => panic!("Expected validation error"),
        },
    }
}

#[test]
fn comps_and_sum() {
    let db = TestProject::new();
    let connection = db.get_connection();
    let hold1 = db.create_hold().with_hold_type(HoldTypes::Comp).finish();
    let hold2 = db.create_hold().with_hold_type(HoldTypes::Comp).finish();
    let comp1 = db
        .create_comp()
        .with_hold(&hold1)
        .with_quantity(3)
        .with_name("Comp1".into())
        .finish();
    let comp2 = db
        .create_comp()
        .with_hold(&hold1)
        .with_quantity(1)
        .with_name("Comp2".into())
        .finish();
    let _comp3 = db
        .create_comp()
        .with_hold(&hold2)
        .with_name("Comp3".into())
        .finish();

    let update_patch = UpdateHoldAttributes {
        hold_type: Some(HoldTypes::Discount.to_string()),
        discount_in_cents: Some(Some(0)),
        ..Default::default()
    };
    let hold2 = hold2.update(update_patch, connection).unwrap();

    let found_comps = hold1.comps(connection).unwrap();
    assert_eq!(vec![comp1, comp2], found_comps);
    assert_eq!(4, hold1.comps_sum(connection).unwrap());

    let found_comps = hold2.comps(connection);
    assert!(found_comps.is_err());
    assert_eq!(0, hold2.comps_sum(connection).unwrap());
}

#[test]
fn set_quantity() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    let hold = db.create_hold().with_event(&event).finish();
    hold.set_quantity(30, db.get_connection()).unwrap();

    assert_eq!(hold.quantity(db.get_connection()).unwrap(), (30, 30));
}

#[test]
fn set_quantity_with_validation_errors() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    let hold = db
        .create_hold()
        .with_hold_type(HoldTypes::Comp)
        .with_event(&event)
        .finish();
    // Initial value of 30
    let conn = db.get_connection();
    hold.set_quantity(30, conn).unwrap();
    assert_eq!(hold.quantity(conn).unwrap(), (30, 30));

    // Comp taking 29 of the hold allows a set quantity of 29 still
    db.create_comp().with_hold(&hold).with_quantity(29).finish();
    hold.set_quantity(29, conn).unwrap();
    assert_eq!(hold.quantity(conn).unwrap(), (29, 29));

    // Fails to set quantity to 28 which would be below comp size
    let result = hold.set_quantity(28, conn);
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("quantity"));
                assert_eq!(errors["quantity"].len(), 1);
                assert_eq!(
                    errors["quantity"][0].code,
                    "assigned_comp_count_greater_than_quantity"
                );
            }
            _ => panic!("Expected validation error"),
        },
    }
}

#[test]
fn organization() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let event = project.create_event().with_ticket_pricing().finish();
    let hold = project.create_hold().with_event(&event).finish();

    let organization = hold.organization(connection).unwrap();
    assert_eq!(event.organization(connection).unwrap(), organization);
}

#[test]
fn find() {
    let db = TestProject::new();
    let hold = db.create_hold().finish();

    let db_hold = Hold::find(hold.id, db.get_connection()).unwrap();

    assert_eq!(hold, db_hold);
}

#[test]
fn find_for_event() {
    let db = TestProject::new();
    let event = db.create_event().with_ticket_pricing().finish();
    let hold = db.create_hold().with_event(&event).finish();
    let hold2 = db.create_hold().with_event(&event).finish();

    let holds = Hold::find_for_event(event.id, db.get_connection()).unwrap();

    assert_eq!(vec![hold, hold2], holds);
}
