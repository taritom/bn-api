use actix_web::{http, test};
use bigneon_api::controllers::artists;
use bigneon_api::database::ConnectionGranting;
use bigneon_api::server::AppState;
use bigneon_db::models::Artist;
use support::database::TestDatabase;

#[test]
fn index() {
    let database = TestDatabase::new();
    let name = "Name";
    let artist = Artist::create(&name)
        .commit(&*database.get_connection())
        .unwrap();

    let response = test::TestRequest::with_state(AppState {
        database: Box::new(database),
    }).run(artists::index)
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);
    assert_eq!(format!("{:?}", response.body()), "test");
}
