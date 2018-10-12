use bigneon_db::models::{DomainEvent, DomainEventTypes, Tables};
use support::project::TestProject;
use uuid::Uuid;

#[test]
fn find() {
    let project = TestProject::new();
    let connection = project.get_connection();
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
        Some("".into()),
    ).commit(connection)
    .unwrap();

    let domain_event2 = DomainEvent::create(
        DomainEventTypes::PaymentMethodUpdated,
        "Payment method was updated".to_string(),
        Tables::PaymentMethods,
        Some(id),
        Some("".into()),
    ).commit(connection)
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
    assert!(
        DomainEvent::find(
            Tables::PaymentMethods,
            Some(Uuid::new_v4()),
            None,
            connection,
        ).unwrap()
        .is_empty()
    );

    // Filtered by type
    assert_eq!(
        DomainEvent::find(
            Tables::PaymentMethods,
            Some(id),
            Some(DomainEventTypes::PaymentMethodCreated),
            connection,
        ).unwrap(),
        [domain_event]
    );
    assert_eq!(
        DomainEvent::find(
            Tables::PaymentMethods,
            Some(id),
            Some(DomainEventTypes::PaymentMethodUpdated),
            connection,
        ).unwrap(),
        [domain_event2]
    );
}
