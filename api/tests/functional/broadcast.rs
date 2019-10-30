use support::database::TestDatabase;
use support::{self, *};
use uuid::Uuid;

#[test]
fn broadcast_counter() {

    let database = TestDatabase::new();
    let connection = database.connection.get();
    let id =  Uuid::new();
    Broadcast::increment_sent_count(, &connection)?;

}