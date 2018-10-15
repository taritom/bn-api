use bigneon_db::models::{
    DomainEvent, DomainEventTypes, PaymentMethod, PaymentMethodEditableAttributes, Tables,
};
use support::project::TestProject;

#[test]
fn create() {
    let project = TestProject::new();
    let user = project.create_user().finish();
    let connection = project.get_connection();

    let payment_method = PaymentMethod::create(
        user.id,
        "stripe".into(),
        true,
        "cus_example".into(),
        "abc".into(),
    ).commit(connection)
    .unwrap();

    let domain_events = DomainEvent::find(
        Tables::PaymentMethods,
        Some(payment_method.id),
        Some(DomainEventTypes::PaymentMethodCreated),
        connection,
    ).unwrap();

    assert!(domain_events.len() == 1);
    assert_eq!(
        domain_events[0].event_data,
        Some(payment_method.provider_data)
    );
}

#[test]
fn update() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let payment_method = project.create_payment_method().finish();
    assert_eq!(
        payment_method.provider_data.to_string(),
        "\"abc\"".to_string()
    );

    assert!(
        DomainEvent::find(
            Tables::PaymentMethods,
            Some(payment_method.id),
            Some(DomainEventTypes::PaymentMethodUpdated),
            connection,
        ).unwrap()
        .is_empty()
    );

    let payment_method_parameters = PaymentMethodEditableAttributes {
        provider_data: Some("test".into()),
    };
    let updated_payment_method = payment_method
        .update(&payment_method_parameters, &project.get_connection())
        .unwrap();

    assert_eq!(
        updated_payment_method.provider_data.to_string(),
        "\"test\"".to_string(),
    );

    let domain_events = DomainEvent::find(
        Tables::PaymentMethods,
        Some(payment_method.id),
        Some(DomainEventTypes::PaymentMethodUpdated),
        connection,
    ).unwrap();

    assert!(domain_events.len() == 1);
    assert_eq!(
        domain_events[0].event_data,
        Some(payment_method.provider_data)
    );
}

#[test]
fn find_default_for_user() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let user3 = project.create_user().finish();
    let user4 = project.create_user().finish();
    let _payment_method = project
        .create_payment_method()
        .with_name("Method1".into())
        .with_user(&user)
        .finish();
    let payment_method2 = project
        .create_payment_method()
        .with_name("Method2".into())
        .with_user(&user)
        .make_default()
        .finish();
    let payment_method3 = project
        .create_payment_method()
        .with_name("Method1".into())
        .with_user(&user2)
        .make_default()
        .finish();
    let _payment_method4 = project
        .create_payment_method()
        .with_name("Method1".into())
        .with_user(&user3)
        .finish();

    // User with multiple methods with default
    let default_payment_method =
        PaymentMethod::find_default_for_user(user.id, &connection).unwrap();
    assert_eq!(payment_method2, default_payment_method);

    // User has one method and is default
    let default_payment_method =
        PaymentMethod::find_default_for_user(user2.id, &connection).unwrap();
    assert_eq!(payment_method3, default_payment_method);

    // User has method but is not set as default
    assert!(PaymentMethod::find_default_for_user(user3.id, &connection).is_err());

    // User has no method
    assert!(PaymentMethod::find_default_for_user(user4.id, &connection).is_err());
}

#[test]
fn find_for_user() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let user3 = project.create_user().finish();
    let payment_method = project
        .create_payment_method()
        .with_name("Method1".into())
        .with_user(&user)
        .finish();
    let payment_method2 = project
        .create_payment_method()
        .with_name("Method2".into())
        .with_user(&user)
        .finish();
    let payment_method3 = project
        .create_payment_method()
        .with_name("Method1".into())
        .with_user(&user2)
        .finish();

    // No name specified
    let found_payment_methods = PaymentMethod::find_for_user(user.id, None, &connection).unwrap();
    assert_eq!(
        vec![payment_method.clone(), payment_method2.clone()],
        found_payment_methods,
    );
    let found_payment_methods = PaymentMethod::find_for_user(user2.id, None, &connection).unwrap();
    assert_eq!(vec![payment_method3.clone()], found_payment_methods);
    let found_payment_methods = PaymentMethod::find_for_user(user3.id, None, &connection).unwrap();
    assert!(found_payment_methods.is_empty());

    // Using specific names
    let found_payment_methods =
        PaymentMethod::find_for_user(user.id, Some("Method1".into()), &connection).unwrap();
    assert_eq!(vec![payment_method.clone()], found_payment_methods);
    let found_payment_methods =
        PaymentMethod::find_for_user(user2.id, Some("Method1".into()), &connection).unwrap();
    assert_eq!(vec![payment_method3.clone()], found_payment_methods);
    let found_payment_methods =
        PaymentMethod::find_for_user(user3.id, Some("Method1".into()), &connection).unwrap();
    assert!(found_payment_methods.is_empty());

    let found_payment_methods =
        PaymentMethod::find_for_user(user.id, Some("Method2".into()), &connection).unwrap();
    assert_eq!(vec![payment_method2.clone()], found_payment_methods);
    let found_payment_methods =
        PaymentMethod::find_for_user(user2.id, Some("Method2".into()), &connection).unwrap();
    assert!(found_payment_methods.is_empty());
    let found_payment_methods =
        PaymentMethod::find_for_user(user3.id, Some("Method2".into()), &connection).unwrap();
    assert!(found_payment_methods.is_empty());
}
