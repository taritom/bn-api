use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::utils::errors::ErrorCode::ValidationError;

#[test]
pub fn create() {
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
pub fn create_with_validation_errors() {
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
}

#[test]
pub fn update() {
    let db = TestProject::new();
    let hold = db.create_hold().finish();

    let update_patch = UpdateHoldAttributes {
        discount_in_cents: Some(Some(10)),
        max_per_order: Some(None),
        end_at: Some(None),
        name: Some("New name".to_string()),
        hold_type: None,
    };
    let new_hold = hold.update(update_patch, db.get_connection()).unwrap();
    assert_eq!(new_hold.name, "New name".to_string());
    assert_eq!(new_hold.max_per_order, None);
    assert_eq!(new_hold.end_at, None);
    assert_eq!(new_hold.discount_in_cents, Some(10));
}

#[test]
pub fn update_with_validation_errors() {
    let db = TestProject::new();
    let hold = db.create_hold().with_hold_type(HoldTypes::Comp).finish();
    assert!(hold.discount_in_cents.is_none());

    let update_patch = UpdateHoldAttributes {
        hold_type: Some(HoldTypes::Discount.to_string()),
        ..Default::default()
    };
    let result = hold.update(update_patch, db.get_connection());
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
}

#[test]
pub fn comps_and_sum() {
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
pub fn set_quantity() {
    let db = TestProject::new();
    let event = db.create_event().with_tickets().finish();
    let hold = db.create_hold().with_event(&event).finish();
    hold.set_quantity(30, db.get_connection()).unwrap();

    assert_eq!(hold.quantity(db.get_connection()).unwrap(), 30);
}

#[test]
pub fn set_quantity_with_validation_errors() {
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
    assert_eq!(hold.quantity(conn).unwrap(), 30);

    // Comp taking 29 of the hold allows a set quantity of 29 still
    db.create_comp().with_hold(&hold).with_quantity(29).finish();
    hold.set_quantity(29, conn).unwrap();
    assert_eq!(hold.quantity(conn).unwrap(), 29);

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
pub fn find() {
    let db = TestProject::new();
    let hold = db.create_hold().finish();

    let db_hold = Hold::find(hold.id, db.get_connection()).unwrap();

    assert_eq!(hold, db_hold);
}

#[test]
pub fn find_for_event() {
    let db = TestProject::new();
    let event = db.create_event().with_ticket_pricing().finish();
    let hold = db.create_hold().with_event(&event).finish();
    let hold2 = db.create_hold().with_event(&event).finish();

    let holds = Hold::find_for_event(event.id, db.get_connection()).unwrap();

    assert_eq!(vec![hold, hold2], holds);
}
