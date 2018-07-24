use actix_web::{http::StatusCode, FromRequest, Json, Path};
use bigneon_api::controllers::venues::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{NewVenue, Organization, OrganizationVenue, User, Venue};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

#[test]
fn index() {
    let database = TestDatabase::new();
    let mut venue = Venue::create(&"Venue")
        .commit(&*database.get_connection())
        .unwrap();
    venue.address = Some(<String>::from("Test Address"));
    venue.city = Some(<String>::from("Test Address"));
    venue.state = Some(<String>::from("Test state"));
    venue.country = Some(<String>::from("Test country"));
    venue.zip = Some(<String>::from("0124"));
    venue.phone = Some(<String>::from("+27123456789"));
    venue = Venue::update(&venue, &*database.get_connection()).unwrap();
    let mut venue2 = Venue::create(&"Venue 2")
        .commit(&*database.get_connection())
        .unwrap();
    venue2.address = Some(<String>::from("Test Address"));
    venue2.city = Some(<String>::from("Test Address"));
    venue2.state = Some(<String>::from("Test state"));
    venue2.country = Some(<String>::from("Test country"));
    venue2.zip = Some(<String>::from("0124"));
    venue2.phone = Some(<String>::from("+27123456789"));
    venue2 = Venue::update(&venue2, &*database.get_connection()).unwrap();

    let expected_venues = vec![venue, venue2];
    let venue_expected_json = serde_json::to_string(&expected_venues).unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let response = venues::index(state);

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, venue_expected_json);
}

#[test]
fn show() {
    let database = TestDatabase::new();
    let mut venue = Venue::create(&"testVenue")
        .commit(&*database.get_connection())
        .unwrap();
    venue.address = Some(<String>::from("Test Address"));
    venue.city = Some(<String>::from("Test Address"));
    venue.state = Some(<String>::from("Test state"));
    venue.country = Some(<String>::from("Test country"));
    venue.zip = Some(<String>::from("0124"));
    venue.phone = Some(<String>::from("+27123456789"));
    let venue_expected_json = serde_json::to_string(&venue).unwrap();

    let _updated_venue = Venue::update(&venue, &*database.get_connection()).unwrap();
    let test_request = TestRequest::create_with_route(
        database,
        &"/venues/{id}",
        &format!("/venues/{}", venue.id.to_string()),
    );
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();

    let response = venues::show((state, path));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, venue_expected_json);
}

#[test]
fn create() {
    let database = TestDatabase::new();
    let name = "Venue Example";

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(NewVenue {
        name: name.clone().to_string(),
    });
    let response = venues::create((state, json));

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(venue.name, name);
}

#[test]
fn update() {
    let database = TestDatabase::new();
    let mut venue = Venue::create("NewVenue")
        .commit(&*database.get_connection())
        .unwrap();
    venue.address = Some(<String>::from("Test Address"));
    venue.city = Some(<String>::from("Test Address"));
    venue.state = Some(<String>::from("Test state"));
    venue.country = Some(<String>::from("Test country"));
    venue.zip = Some(<String>::from("0124"));
    venue.phone = Some(<String>::from("+27123456789"));
    let _updated_venue = Venue::update(&venue, &*database.get_connection()).unwrap();
    let new_name = "New Name";

    let test_request = TestRequest::create_with_route(
        database,
        &"/venues/{id}",
        &format!("/venues/{}", venue.id.to_string()),
    );
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    let json = Json(Venue {
        id: venue.id,
        address: venue.address.clone(),
        city: venue.city.clone(),
        state: venue.state.clone(),
        country: venue.country.clone(),
        zip: venue.zip.clone(),
        phone: venue.phone.clone(),
        name: new_name.clone().to_string(),
    });

    let response = venues::update((state, path, json));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_venue.name, new_name);
}

#[test]
fn show_from_organizations() {
    let database = TestDatabase::new();
    //create user
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    //create organization
    let mut organization = Organization::create(user.id, &"testOrganization")
        .commit(&*database.get_connection())
        .unwrap();
    organization.address = Some(<String>::from("Test Address"));
    organization.city = Some(<String>::from("Test Address"));
    organization.state = Some(<String>::from("Test state"));
    organization.country = Some(<String>::from("Test country"));
    organization.zip = Some(<String>::from("0124"));
    organization.phone = Some(<String>::from("+27123456789"));
    organization = Organization::update(&organization, &*database.get_connection()).unwrap();
    //create venue
    let mut venue = Venue::create("NewVenue")
        .commit(&*database.get_connection())
        .unwrap();
    venue.address = Some(<String>::from("Test Address"));
    venue.city = Some(<String>::from("Test Address"));
    venue.state = Some(<String>::from("Test state"));
    venue.country = Some(<String>::from("Test country"));
    venue.zip = Some(<String>::from("0124"));
    venue.phone = Some(<String>::from("+27123456789"));
    let updated_venue = Venue::update(&venue, &*database.get_connection()).unwrap();
    let mut venue2 = Venue::create("NewVenue2")
        .commit(&*database.get_connection())
        .unwrap();
    venue2.address = Some(<String>::from("Test Address"));
    venue2.city = Some(<String>::from("Test Address"));
    venue2.state = Some(<String>::from("Test state"));
    venue2.country = Some(<String>::from("Test country"));
    venue2.zip = Some(<String>::from("0124"));
    venue2.phone = Some(<String>::from("+27123456789"));
    let updated_venue2 = Venue::update(&venue2, &*database.get_connection()).unwrap();
    //link venues to organization
    //Do linking
    let _org_venue_link =
        updated_venue.add_to_organization(&organization.id, &*database.get_connection());
    let _org_venue_link =
        updated_venue2.add_to_organization(&organization.id, &*database.get_connection());

    let all_venues = vec![updated_venue, updated_venue2];
    let venue_expected_json = serde_json::to_string(&all_venues).unwrap();
    //find venue from organization
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let json = Json(organization.id);
    let response = venues::show_from_organizations((state, json));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(venue_expected_json, body);
}
#[test]
fn add_to_organization() {
    let database = TestDatabase::new();
    //create user
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    //create organization
    let mut organization = Organization::create(user.id, &"testOrganization")
        .commit(&*database.get_connection())
        .unwrap();
    organization.address = Some(<String>::from("Test Address"));
    organization.city = Some(<String>::from("Test Address"));
    organization.state = Some(<String>::from("Test state"));
    organization.country = Some(<String>::from("Test country"));
    organization.zip = Some(<String>::from("0124"));
    organization.phone = Some(<String>::from("+27123456789"));
    let _updated_organization =
        Organization::update(&organization, &*database.get_connection()).unwrap();
    //create venue
    let mut venue = Venue::create("NewVenue")
        .commit(&*database.get_connection())
        .unwrap();
    venue.address = Some(<String>::from("Test Address"));
    venue.city = Some(<String>::from("Test Address"));
    venue.state = Some(<String>::from("Test state"));
    venue.country = Some(<String>::from("Test country"));
    venue.zip = Some(<String>::from("0124"));
    venue.phone = Some(<String>::from("+27123456789"));
    Venue::update(&venue, &*database.get_connection()).unwrap();

    //link venues to organization
    let test_request = TestRequest::create_with_route(
        database,
        &"/venues/{id}",
        &format!("/venues/{}", venue.id.to_string()),
    );
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    let json = Json(organization.id);
    let response = venues::add_to_organization((state, path, json));

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let organization_venue: OrganizationVenue = serde_json::from_str(&body).unwrap();
    assert_eq!(organization_venue.organization_id, organization.id);
    assert_eq!(organization_venue.venue_id, venue.id);
}
