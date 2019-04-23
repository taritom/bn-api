use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::utils::errors::DatabaseError;
use bigneon_db::utils::errors::ErrorCode::{self, ValidationError};
use chrono::prelude::*;
use diesel;
use diesel::sql_types;
use diesel::RunQueryDsl;
use uuid::Uuid;

#[test]
fn find_active_pending_by_ticket_instance_ids() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    project
        .create_order()
        .for_user(&user)
        .quantity(3)
        .is_paid()
        .finish();
    let tickets = TicketInstance::find_for_user(user.id, connection).unwrap();
    let ticket = &tickets[0];
    let ticket2 = &tickets[1];
    let ticket3 = &tickets[2];
    let transfer_key = Uuid::new_v4();
    let transfer_expiry_date = NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11);

    let transfer = Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();
    assert!(transfer.complete(user2.id, None, connection).is_ok());
    let transfer2 = Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();
    let transfer3 = Transfer::create(
        ticket2.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();

    // Transfer 3 is expired so will not returned
    let _q: Vec<TicketInstance> = diesel::sql_query(
        r#"
        UPDATE transfers
        SET transfer_expiry_date = '2018-06-06 09:49:09.643207'
        WHERE id = $1;
        "#,
    )
    .bind::<sql_types::Uuid, _>(transfer3.id)
    .get_results(connection)
    .unwrap();

    let pending_transfers = Transfer::find_active_pending_by_ticket_instance_ids(
        &[ticket.id, ticket2.id, ticket3.id],
        connection,
    )
    .unwrap();
    assert_eq!(pending_transfers.len(), 1);
    assert_eq!(pending_transfers[0].id, transfer2.id);
}

#[test]
fn cancel() {
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
    let transfer_key = Uuid::new_v4();
    let transfer_expiry_date = NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11);
    let transfer = Transfer::create(ticket.id, user.id, transfer_key, transfer_expiry_date)
        .commit(None, connection)
        .unwrap();

    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketCancelled),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());
    let domain_events = DomainEvent::find(
        Tables::Transfers,
        Some(transfer.id),
        Some(DomainEventTypes::TransferTicketCancelled),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());

    let transfer = transfer.cancel(user.id, None, connection).unwrap();
    assert_eq!(transfer.status, TransferStatus::Cancelled);
    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketCancelled),
        connection,
    )
    .unwrap();
    assert_eq!(1, domain_events.len());
    let domain_events = DomainEvent::find(
        Tables::Transfers,
        Some(transfer.id),
        Some(DomainEventTypes::TransferTicketCancelled),
        connection,
    )
    .unwrap();
    assert_eq!(1, domain_events.len());

    // Transfering again triggers error as status is no longer pending
    let result = transfer.cancel(user.id, None, connection);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        DatabaseError::new(
            ErrorCode::UpdateError,
            Some("Transfer cannot be cancelled as it is no longer pending".to_string()),
        )
    );
}

#[test]
fn complete() {
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
    let transfer_key = Uuid::new_v4();
    let transfer_expiry_date = NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11);
    let transfer = Transfer::create(ticket.id, user.id, transfer_key, transfer_expiry_date)
        .commit(None, connection)
        .unwrap();

    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketCompleted),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());
    let domain_events = DomainEvent::find(
        Tables::Transfers,
        Some(transfer.id),
        Some(DomainEventTypes::TransferTicketCompleted),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());

    let transfer = transfer.complete(user2.id, None, connection).unwrap();
    assert_eq!(transfer.status, TransferStatus::Completed);
    assert_eq!(transfer.destination_user_id, Some(user2.id));
    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketCompleted),
        connection,
    )
    .unwrap();
    assert_eq!(1, domain_events.len());
    let domain_events = DomainEvent::find(
        Tables::Transfers,
        Some(transfer.id),
        Some(DomainEventTypes::TransferTicketCompleted),
        connection,
    )
    .unwrap();
    assert_eq!(1, domain_events.len());

    // Transfering again triggers error as status is no longer pending
    let result = transfer.complete(user2.id, None, connection);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        DatabaseError::new(
            ErrorCode::UpdateError,
            Some("Transfer cannot be completed as it is no longer pending".to_string()),
        )
    );
}

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
    let transfer_key = Uuid::new_v4();
    let transfer_expiry_date = NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11);
    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketStarted),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());

    let transfer = Transfer::create(ticket.id, user.id, transfer_key, transfer_expiry_date)
        .commit(None, connection)
        .unwrap();
    assert_eq!(transfer.status, TransferStatus::Pending);
    assert_eq!(transfer.ticket_instance_id, ticket.id);
    assert_eq!(transfer.source_user_id, user.id);
    assert_eq!(transfer.transfer_key, transfer_key);
    assert_eq!(
        transfer.transfer_expiry_date.timestamp(),
        transfer_expiry_date.timestamp()
    );

    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketStarted),
        connection,
    )
    .unwrap();
    assert_eq!(1, domain_events.len());

    let domain_events = DomainEvent::find(
        Tables::Transfers,
        Some(transfer.id),
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
    let transfer_key = Uuid::new_v4();
    let transfer_expiry_date = NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11);
    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketStarted),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());

    let transfer = Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();

    // Active pending transfer already exists triggering validation errors
    let result = Transfer::create(ticket.id, user.id, transfer_key, transfer_expiry_date)
        .commit(None, connection);
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
    let transfer = Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();

    // Pending but expired transfer does not cause an issue
    assert!(Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date
    )
    .commit(None, connection)
    .is_err());
    let _q: Vec<TicketInstance> = diesel::sql_query(
        r#"
        UPDATE transfers
        SET transfer_expiry_date = '2018-06-06 09:49:09.643207'
        WHERE id = $1;
        "#,
    )
    .bind::<sql_types::Uuid, _>(transfer.id)
    .get_results(connection)
    .unwrap();
    assert!(Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date
    )
    .commit(None, connection)
    .is_ok());
}

#[test]
fn update() {
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
    let transfer_key = Uuid::new_v4();
    let transfer_expiry_date = NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11);
    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketStarted),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());

    let transfer = Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();
    let transfer = transfer
        .update(
            TransferEditableAttributes {
                status: Some(TransferStatus::Cancelled),
                ..Default::default()
            },
            connection,
        )
        .unwrap();
    assert_eq!(transfer.status, TransferStatus::Cancelled);
}

#[test]
fn update_with_validation_error() {
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
    let transfer_key = Uuid::new_v4();
    let transfer_expiry_date = NaiveDate::from_ymd(2050, 7, 8).and_hms(4, 10, 11);
    let domain_events = DomainEvent::find(
        Tables::TicketInstances,
        Some(ticket.id),
        Some(DomainEventTypes::TransferTicketStarted),
        connection,
    )
    .unwrap();
    assert_eq!(0, domain_events.len());

    let transfer = Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();
    let transfer = transfer
        .update(
            TransferEditableAttributes {
                status: Some(TransferStatus::Cancelled),
                ..Default::default()
            },
            connection,
        )
        .unwrap();
    let transfer2 = Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();

    let result = transfer.update(
        TransferEditableAttributes {
            status: Some(TransferStatus::Pending),
            ..Default::default()
        },
        connection,
    );

    assert!(result.is_err());
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

    // Complete second transfer making first eligible to be updated to this state
    assert!(transfer2.complete(user.id, None, connection).is_ok());
    let transfer = Transfer::create(
        ticket.id,
        user.id,
        transfer_key.clone(),
        transfer_expiry_date,
    )
    .commit(None, connection)
    .unwrap();
    assert!(transfer
        .update(
            TransferEditableAttributes {
                status: Some(TransferStatus::Pending),
                ..Default::default()
            },
            connection
        )
        .is_ok());
}
