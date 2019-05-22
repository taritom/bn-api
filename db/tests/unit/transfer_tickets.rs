use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::utils::errors::ErrorCode::ValidationError;
use chrono::prelude::*;
use diesel;
use diesel::sql_types;
use diesel::RunQueryDsl;
use uuid::Uuid;

#[test]
fn create_commit() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    project
        .create_order()
        .for_user(&user)
        .quantity(1)
        .is_paid()
        .finish();
    let ticket = TicketInstance::find_for_user(user.id, connection)
        .unwrap()
        .remove(0);
    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketStarted),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());

    let transfer = Transfer::create(
        user.id,
        Uuid::new_v4(),
        NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11),
        None,
        None,
    )
    .commit(&None, connection)
    .unwrap();
    let transfer_ticket = TransferTicket::create(ticket.id, transfer.id)
        .commit(user.id, &None, connection)
        .unwrap();
    assert_eq!(transfer_ticket.ticket_instance_id, ticket.id);
    assert_eq!(transfer_ticket.transfer_id, transfer.id);

    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketStarted),
        connection,
    )
    .unwrap();
    assert_eq!(1, domain_events.len());
}

#[test]
fn create_commit_with_validation_error() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    project
        .create_order()
        .for_user(&user)
        .quantity(1)
        .is_paid()
        .finish();
    let ticket = TicketInstance::find_for_user(user.id, connection)
        .unwrap()
        .remove(0);
    let transfer_expiry_date = NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11);
    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketStarted),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());

    let transfer = Transfer::create(user.id, Uuid::new_v4(), transfer_expiry_date, None, None)
        .commit(&None, connection)
        .unwrap();
    TransferTicket::create(ticket.id, transfer.id)
        .commit(user.id, &None, connection)
        .unwrap();

    // Active pending transfer already exists triggering validation errors
    let transfer2 = Transfer::create(user.id, Uuid::new_v4(), transfer_expiry_date, None, None)
        .commit(&None, connection)
        .unwrap();
    let result = TransferTicket::create(ticket.id, transfer2.id).commit(user.id, &None, connection);
    assert!(result.is_err());
    print!("{:?}", result);
    match result {
        Ok(_) => {
            panic!("Expected validation error");
        }
        Err(error) => match &error.error_code {
            ValidationError { errors } => {
                assert!(errors.contains_key("ticket_instance_id"));
                assert_eq!(errors["ticket_instance_id"].len(), 1);
                assert_eq!(
                    errors["ticket_instance_id"][0].code,
                    "too_many_pending_transfers"
                );
                assert_eq!(
                    errors["ticket_instance_id"][0]
                        .message
                        .clone()
                        .unwrap()
                        .into_owned(),
                    "An active pending transfer already exists for this ticket instance id"
                )
            }
            _ => panic!("Expected validation error"),
        },
    }

    // Original transfer is now complete so no issue
    assert!(transfer.complete(user2.id, None, connection).is_ok());
    TransferTicket::create(ticket.id, transfer2.id)
        .commit(user.id, &None, connection)
        .unwrap();

    let transfer3 = Transfer::create(user.id, Uuid::new_v4(), transfer_expiry_date, None, None)
        .commit(&None, connection)
        .unwrap();

    // Pending but expired transfer does not cause an issue

    assert!(TransferTicket::create(ticket.id, transfer3.id)
        .commit(user.id, &None, connection)
        .is_err());
    let _q: Vec<TicketInstance> = diesel::sql_query(
        r#"
        UPDATE transfers
        SET transfer_expiry_date = '2018-06-06 09:49:09.643207'
        WHERE id = $1;
        "#,
    )
    .bind::<sql_types::Uuid, _>(transfer2.id)
    .get_results(connection)
    .unwrap();
    assert!(TransferTicket::create(ticket.id, transfer3.id)
        .commit(user.id, &None, connection)
        .is_ok());
}
