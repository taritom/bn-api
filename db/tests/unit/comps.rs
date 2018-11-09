use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::utils::errors::ErrorCode::ValidationError;
use uuid::Uuid;

#[test]
fn create() {
    let db = TestProject::new();
    let hold = db.create_hold().with_hold_type(HoldTypes::Comp).finish();
    Comp::create(
        "test".into(),
        hold.id,
        Some("email@address.com".into()),
        None,
        5,
    ).commit(db.get_connection())
    .unwrap();
}

#[test]
pub fn create_with_validation_errors() {
    let db = TestProject::new();
    let hold = db
        .create_hold()
        .with_hold_type(HoldTypes::Discount)
        .finish();
    let result = Comp::create("test".into(), hold.id, Some("invalid".into()), None, 11)
        .commit(db.get_connection());

    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("email"));
                assert_eq!(errors["email"].len(), 1);
                assert_eq!(errors["email"][0].code, "email");

                assert!(errors.contains_key("hold_id"));
                assert_eq!(errors["hold_id"].len(), 1);
                assert_eq!(
                    errors["hold_id"][0].code,
                    "comps_hold_type_valid_for_comp_creation"
                );

                assert!(errors.contains_key("quantity"));
                assert_eq!(errors["quantity"].len(), 1);
                assert_eq!(
                    errors["quantity"][0].code,
                    "comps_quantity_valid_for_hold_quantity"
                );
            }
            _ => panic!("Expected validation error"),
        },
    }
}

#[test]
fn update() {
    let db = TestProject::new();
    let comp = db.create_comp().finish();

    let update_patch = UpdateCompAttributes {
        name: Some("New name".to_string()),
        email: Some("new@email.com".to_string()),
        ..Default::default()
    };
    let new_comp = comp.update(update_patch, db.get_connection()).unwrap();
    assert_eq!(new_comp.name, "New name".to_string());
    assert_eq!(new_comp.email, Some("new@email.com".to_string()));
}

#[test]
pub fn update_with_validation_errors() {
    let db = TestProject::new();
    let comp = db.create_comp().finish();

    let update_patch = UpdateCompAttributes {
        quantity: Some(11),
        email: Some("invalid".to_string()),
        ..Default::default()
    };

    let result = comp.update(update_patch, db.get_connection());
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("email"));
                assert_eq!(errors["email"].len(), 1);
                assert_eq!(errors["email"][0].code, "email");

                assert!(errors.contains_key("quantity"));
                assert_eq!(errors["quantity"].len(), 1);
                assert_eq!(
                    errors["quantity"][0].code,
                    "comps_quantity_valid_for_hold_quantity"
                );
            }
            _ => panic!("Expected validation error"),
        },
    }
}

#[test]
fn find_by_redemption_code() {
    let db = TestProject::new();
    let connection = db.get_connection();
    let comp = db.create_comp().finish();
    let found_comp = Comp::find_by_redemption_code(&comp.redemption_code, connection).unwrap();
    assert_eq!(comp, found_comp);
}

#[test]
fn find_for_hold() {
    let db = TestProject::new();
    let connection = db.get_connection();
    let hold1 = db.create_hold().with_hold_type(HoldTypes::Comp).finish();
    let hold2 = db.create_hold().with_hold_type(HoldTypes::Comp).finish();
    let hold3 = db.create_hold().with_hold_type(HoldTypes::Comp).finish();
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

    let found_comps = Comp::find_for_hold(hold1.id, connection).unwrap();
    assert_eq!(vec![comp1, comp2], found_comps);
    assert_eq!(4, Comp::sum_for_hold(hold1.id, connection).unwrap());

    let found_comps = Comp::find_for_hold(hold2.id, connection).unwrap();
    assert!(found_comps.is_empty());
    assert_eq!(0, Comp::sum_for_hold(hold2.id, connection).unwrap());

    let found_comps = Comp::find_for_hold(hold3.id, connection).unwrap();
    assert!(found_comps.is_empty());
    assert_eq!(0, Comp::sum_for_hold(hold3.id, connection).unwrap());
}

#[test]
fn find() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let comp = project.create_comp().finish();

    // Record found
    let found_comp = Comp::find(comp.hold_id, comp.id, connection).unwrap();
    assert_eq!(comp, found_comp);

    // Comp does not exist for hold so returns error
    assert!(Comp::find(Uuid::new_v4(), comp.id, connection).is_err());
}

#[test]
fn destroy() {
    let project = TestProject::new();
    let comp = project.create_comp().finish();
    assert!(comp.destroy(project.get_connection()).unwrap() > 0);
    assert!(Comp::find(comp.hold_id, comp.id, project.get_connection()).is_err());
}

#[test]
fn destroy_from_hold() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let hold = project
        .create_hold()
        .with_hold_type(HoldTypes::Comp)
        .finish();
    let comp = project.create_comp().with_hold(&hold).finish();
    let comp2 = project.create_comp().with_hold(&hold).finish();
    let comp3 = project.create_comp().finish();
    assert_eq!(Comp::destroy_from_hold(hold.id, connection).unwrap(), 2);
    assert!(Comp::find(comp.hold_id, comp.id, connection).is_err());
    assert!(Comp::find(comp2.hold_id, comp2.id, connection).is_err());
    // Not deleted as part of another hold
    assert!(Comp::find(comp3.hold_id, comp3.id, connection).is_ok());
}
