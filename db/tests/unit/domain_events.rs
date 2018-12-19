use bigneon_db::dev::TestProject;
use bigneon_db::prelude::*;
use uuid::Uuid;

#[test]
fn find() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let id = Uuid::new_v4();

    // Empty, no events
    assert!(
        DomainEvent::find(Tables::PaymentMethods, Some(id), None, connection)
            .unwrap()
            .is_empty()
    );

    // New events
    let domain_event = DomainEvent::create(
        DomainEventTypes::PaymentMethodCreated,
        "Payment method was created".to_string(),
        Tables::PaymentMethods,
        Some(id),
        Some(user.id),
        Some("".into()),
    )
    .commit(connection)
    .unwrap();

    let domain_event2 = DomainEvent::create(
        DomainEventTypes::PaymentMethodUpdated,
        "Payment method was updated".to_string(),
        Tables::PaymentMethods,
        Some(id),
        Some(user.id),
        Some("".into()),
    )
    .commit(connection)
    .unwrap();

    assert_eq!(
        DomainEvent::find(Tables::PaymentMethods, Some(id), None, connection).unwrap(),
        [domain_event.clone(), domain_event2.clone()]
    );
    assert!(
        DomainEvent::find(Tables::Payments, Some(id), None, connection)
            .unwrap()
            .is_empty()
    );
    assert!(DomainEvent::find(
        Tables::PaymentMethods,
        Some(Uuid::new_v4()),
        None,
        connection,
    )
    .unwrap()
    .is_empty());

    // Filtered by type
    assert_eq!(
        DomainEvent::find(
            Tables::PaymentMethods,
            Some(id),
            Some(DomainEventTypes::PaymentMethodCreated),
            connection,
        )
        .unwrap(),
        [domain_event]
    );
    assert_eq!(
        DomainEvent::find(
            Tables::PaymentMethods,
            Some(id),
            Some(DomainEventTypes::PaymentMethodUpdated),
            connection,
        )
        .unwrap(),
        [domain_event2]
    );
}

#[test]
pub fn find_unpublished() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let id = Uuid::new_v4();

    let domain_event = DomainEvent::create(
        DomainEventTypes::PaymentMethodCreated,
        "Payment method was created".to_string(),
        Tables::PaymentMethods,
        Some(id),
        Some(user.id),
        Some("".into()),
    )
    .commit(connection)
    .unwrap();

    let mut found_events = DomainEvent::find_unpublished(100, connection).unwrap();

    let db_event = found_events.remove(0);
    assert_eq!(db_event, domain_event);

    let mut publisher = DomainEventPublisher::new();
    publisher.add_subscription(DomainEventTypes::PaymentMethodCreated, |_| None);
    publisher.publish(db_event, connection).unwrap();

    let found_events = DomainEvent::find_unpublished(100, connection).unwrap();
    assert!(found_events.is_empty());
}
