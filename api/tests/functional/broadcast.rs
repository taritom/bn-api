use bigneon_db::models::enums::{BroadcastChannel, BroadcastType, BroadcastAudience};
use bigneon_db::prelude::Broadcast;
use std::string::ToString;
use support::database::TestDatabase;

#[test]
fn broadcast_counter() {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let id = database.create_event().finish().id;
    let broadcast = Broadcast::create(
        id,
        BroadcastType::Custom,
        BroadcastChannel::PushNotification,
        Option::from("Name".to_string()),
        None,
        None,
        None,
        BroadcastAudience::PeopleAtTheEvent,
    )
    .commit(connection)
    .unwrap();
    Broadcast::increment_sent_count(broadcast.id, &connection).unwrap();
    Broadcast::increment_open_count(broadcast.id, &connection).unwrap();
    let b = Broadcast::find(broadcast.id, &connection).unwrap();
    assert_eq!(b.sent_quantity, 1);
    assert_eq!(b.opened_quantity, 1);
}
