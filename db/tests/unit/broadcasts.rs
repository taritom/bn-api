use bigneon_db::dev::TestProject;
use bigneon_db::models::Scopes::BoxOfficeTicketRead;
use bigneon_db::prelude::*;
use chrono::Utc;

#[test]
fn new_broadcast_commit() {
    let project = TestProject::new();
    let conn = project.get_connection();
    let event = project.create_event().finish();

    let send_at = Utc::now().naive_utc();

    let broadcast = Broadcast::create(
        event.id,
        BroadcastType::LastCall,
        BroadcastChannel::PushNotification,
        "myname".to_string(),
        Some(send_at),
        None,
        None,
        BroadcastAudience::PeopleAtTheEvent,
    );

    assert_eq!(
        BroadcastStatus::Pending,
        broadcast.status,
        "Invalid status for NewBroadcast"
    );

    let broadcast = broadcast.commit(conn).unwrap();
    assert!(!broadcast.id.is_nil());

    assert_eq!(broadcast.channel, BroadcastChannel::PushNotification);

    let domain_actions = DomainAction::find_pending(Some(DomainActionTypes::BroadcastPushNotification), conn).unwrap();
    assert_eq!(domain_actions.len(), 1, "DomainAction was not created");
    assert_eq!(domain_actions[0].main_table_id.unwrap(), broadcast.id);
    assert_eq!(domain_actions[0].scheduled_at.timestamp(), send_at.timestamp());
}

#[test]
fn new_custom_broadcast_commit() {
    let project = TestProject::new();
    let conn = project.get_connection();
    let event = project.create_event().finish();

    let broadcast_err = Broadcast::create(
        event.id,
        BroadcastType::Custom,
        BroadcastChannel::PushNotification,
        Option::from("Custom Name No Message".to_string()),
        None,
        None,
        None,
        BroadcastAudience::PeopleAtTheEvent,
    )
    .commit(conn);
    assert!(broadcast_err.is_err());

    let broadcast = Broadcast::create(
        event.id,
        BroadcastType::Custom,
        BroadcastChannel::PushNotification,
        Option::from("Custom Name".to_string()),
        None,
        None,
        None,
        BroadcastAudience::PeopleAtTheEvent,
    );

    assert_eq!(
        BroadcastStatus::Pending,
        broadcast.status,
        "Invalid status for NewBroadcast"
    );

    let broadcast = broadcast.commit(conn).unwrap();
    assert!(!broadcast.id.is_nil());

    assert_eq!(broadcast.channel, BroadcastChannel::PushNotification);
    assert_eq!(broadcast.message, Some("Custom Message".to_string()));

    let domain_actions = DomainAction::find_pending(Some(DomainActionTypes::BroadcastPushNotification), conn).unwrap();

    assert_eq!(domain_actions.len(), 1, "DomainAction was not created");
    assert_eq!(domain_actions[0].main_table_id.unwrap(), broadcast.id);
}

#[test]
fn broadcast_find() {
    let project = TestProject::new();
    let conn = project.get_connection();

    let broadcast = project.create_broadcast().finish();

    let found = Broadcast::find(broadcast.id, conn).unwrap();
    assert_eq!(broadcast.id, found.id);
}

#[test]
fn broadcast_find_by_id() {
    let project = TestProject::new();
    let conn = project.get_connection();

    let event = project.create_event().finish();
    let broadcast = project.create_broadcast().with_event_id(event.id).finish();

    let found = Broadcast::find_by_event_id(event.id, 0, 1, conn).unwrap();
    assert_eq!(1, found.data.len());
    assert_eq!(broadcast.id, found.data[0].id);
    assert_eq!(0, found.paging.page);
}

#[test]
fn broadcast_cancel() {
    let project = TestProject::new();
    let conn = project.get_connection();

    let broadcast = project.create_broadcast().finish();
    let broadcast = broadcast.cancel(conn).unwrap();

    assert_eq!(broadcast.status, BroadcastStatus::Cancelled);
}

#[test]
fn broadcast_update() {
    let project = TestProject::new();
    let conn = project.get_connection();

    let broadcast = project
        .create_broadcast()
        .with_channel(BroadcastChannel::PushNotification)
        .with_send_at(Utc::now().naive_utc())
        .with_status(BroadcastStatus::Pending)
        .finish();

    let attributes = BroadcastEditableAttributes {
        notification_type: None,
        channel: None,
        message: None,
        send_at: Some(None),
        status: Some(BroadcastStatus::InProgress),
    };

    let broadcast = broadcast.update(attributes, conn).unwrap();

    assert_eq!(broadcast.status, BroadcastStatus::InProgress);
    assert_eq!(broadcast.channel, BroadcastChannel::PushNotification);
    assert!(broadcast.send_at.is_none());
}

#[test]
fn broadcast_update_if_cancelled() {
    let project = TestProject::new();
    let conn = project.get_connection();

    let broadcast = project
        .create_broadcast()
        .with_status(BroadcastStatus::Cancelled)
        .finish();

    let attributes = BroadcastEditableAttributes {
        notification_type: None,
        channel: None,
        message: None,
        send_at: Some(None),
        status: Some(BroadcastStatus::InProgress),
    };

    let error = broadcast.update(attributes, conn).err();
    assert!(error.is_some(), "broadcast.update did not return expected error");
    let error = error.unwrap();
    assert_eq!(error.error_code, ErrorCode::UpdateError);
    assert_eq!(
        "This broadcast has been cancelled, it cannot be modified.",
        error.cause.unwrap()
    );
}

#[test]
fn broadcast_set_in_progress() {
    let project = TestProject::new();
    let conn = project.get_connection();

    let broadcast = project
        .create_broadcast()
        .with_status(BroadcastStatus::Pending)
        .finish();

    let broadcast = broadcast.set_in_progress(conn).unwrap();
    assert_eq!(BroadcastStatus::InProgress, broadcast.status);
}
