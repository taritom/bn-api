use support::database::TestDatabase;
use support;
use bigneon_db::models::{Roles};
use bigneon_db::prelude::*;

#[test]
fn index() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let connection = database.connection.get();
    let organization = database.create_organization().finish();
    let venue = database
        .create_venue()
        .with_city("San Francisco".to_string())
        .with_state("California".to_string())
        .with_country("US".to_string())
        .finish();
    let venue2 = database
        .create_venue()
        .with_city("Oakland".to_string())
        .with_state("California".to_string())
        .with_country("US".to_string())
        .finish();
    let event = database
        .create_event()
        .with_name("NewEvent1".to_string())
        .with_organization(&organization)
        .with_venue(&venue)
        .finish();
    let event2 = database
        .create_event()
        .with_name("NewEvent2".to_string())
        .with_organization(&organization)
        .with_venue(&venue2)
        .finish();

    let slug = database
        .create_slug()
        .for_event(&event)
        .with_slug("redirect-me")
        .finish();
    let slug2 = database
        .create_slug()
        .for_venue(&venue, SlugTypes::Venue)
        .with_slug("redirect-me2")
        .finish();
    let slug = database
        .create_slug()
        .for_organization(&organization)
        .with_slug("redirect-me3")
        .finish();
}
