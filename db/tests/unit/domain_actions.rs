use bigneon_db::dev::TestProject;
use bigneon_db::prelude::*;
use chrono::{Duration, Utc};
use uuid::Uuid;

#[test]
fn has_pending_action() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let id = Uuid::new_v4();

    // Empty, no action
    let result = DomainAction::has_pending_action(
        DomainActionTypes::MarketingContactsBulkEventFanListImport,
        Tables::Events.table_name(),
        id,
        connection,
    )
    .unwrap();
    assert!(!result);

    // New action
    let mut domain_action = DomainAction::create(
        None,
        DomainActionTypes::MarketingContactsBulkEventFanListImport,
        None,
        json!(Vec::<u8>::new()),
        Some(Tables::Events.table_name()),
        Some(id),
    );
    domain_action.scheduled_at = Utc::now()
        .naive_utc()
        .checked_add_signed(Duration::hours(1))
        .unwrap();
    domain_action.expires_at = Utc::now()
        .naive_utc()
        .checked_add_signed(Duration::days(1))
        .unwrap();

    domain_action.commit(connection).unwrap();

    let result = DomainAction::has_pending_action(
        DomainActionTypes::MarketingContactsBulkEventFanListImport,
        Tables::Events.table_name(),
        id,
        connection,
    )
    .unwrap();
    assert!(result);
}
