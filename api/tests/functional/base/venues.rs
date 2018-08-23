use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::venues::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_api::models::AddVenueToOrganizationRequest;
use bigneon_db::models::{
    NewVenue, Organization, OrganizationVenue, Roles, User, Venue, VenueEditableAttributes,
};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

pub fn index(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let venue = (NewVenue {
        name: "New Venue".to_string(),
        address: Some("Address".to_string()),
        city: Some("City".to_string()),
        state: Some("State".to_string()),
        country: Some("Country".to_string()),
        zip: Some("-1234".to_string()),
        phone: Some("+27123456789".to_string()),
    }).commit(&*connection)
        .unwrap();

    let venue2 = (NewVenue {
        name: "New Venue2".to_string(),
        address: Some("Address2".to_string()),
        city: Some("City2".to_string()),
        state: Some("State2".to_string()),
        country: Some("Country2".to_string()),
        zip: None,
        phone: None,
    }).commit(&*connection)
        .unwrap();

    let expected_venues = vec![venue, venue2];
    let venue_expected_json = serde_json::to_string(&expected_venues).unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let response: HttpResponse =
        venues::index((state, support::create_auth_user(role, &*connection))).into();

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, venue_expected_json);
}

pub fn show(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let venue = (NewVenue {
        name: "New Venue".to_string(),
        address: Some("Address".to_string()),
        city: Some("City".to_string()),
        state: Some("State".to_string()),
        country: Some("Country".to_string()),
        zip: Some("-1234".to_string()),
        phone: Some("+27123456789".to_string()),
    }).commit(&*connection)
        .unwrap();
    let venue_expected_json = serde_json::to_string(&venue).unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let response: HttpResponse =
        venues::show((state, path, support::create_auth_user(role, &*connection))).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, venue_expected_json);
}

pub fn show_with_invalid_id(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = Uuid::new_v4();

    let response: HttpResponse =
        venues::show((state, path, support::create_auth_user(role, &*connection))).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

pub fn create(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let name = "Venue Example";

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(NewVenue {
        name: name.clone().to_string(),
        address: None,
        country: None,
        city: None,
        phone: None,
        state: None,
        zip: None,
    });
    let user = support::create_auth_user(role, &*connection);

    let response: HttpResponse = venues::create((state, json, user)).into();

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(venue.name, name);
}

pub fn update(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    let venue = Venue::create("NewVenue").commit(&*connection).unwrap();
    let new_name = "New Name";

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let json = Json(VenueEditableAttributes {
        address: None,
        city: None,
        state: None,
        country: None,
        zip: None,
        phone: None,
        name: Some(new_name.to_string()),
    });

    let user = support::create_auth_user(role, &*connection);

    let response: HttpResponse = venues::update((state, path, json, user)).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_venue.name, new_name);
}

pub fn show_from_organizations(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    //create user
    let user = User::create(
        "Jeff",
        "Jefferson",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*database.get_connection())
        .unwrap();
    //create organization
    let organization = Organization::create(user.id, &"testOrganization")
        .commit(&*database.get_connection())
        .unwrap();
    //create venue
    let venue = Venue::create("NewVenue").commit(&*connection).unwrap();
    let venue2 = Venue::create("NewVenue2").commit(&*connection).unwrap();
    venue
        .add_to_organization(&organization.id, &*connection)
        .unwrap();
    venue2
        .add_to_organization(&organization.id, &*connection)
        .unwrap();

    let all_venues = vec![venue, venue2];
    let venue_expected_json = serde_json::to_string(&all_venues).unwrap();
    //find venue from organization
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let response: HttpResponse = venues::show_from_organizations((
        state,
        path,
        support::create_auth_user(role, &*connection),
    )).into();

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(venue_expected_json, body);
}

pub fn add_to_organization(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    //create user
    let user = User::create(
        "Jeff",
        "Jefferies",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let user = user.add_role(Roles::Admin, &*connection).unwrap();

    //create organization
    let organization = Organization::create(user.id, &"testOrganization")
        .commit(&*connection)
        .unwrap();
    //create venue
    let venue = Venue::create("NewVenue").commit(&*connection).unwrap();

    //link venues to organization
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let json = Json(AddVenueToOrganizationRequest {
        organization_id: organization.id,
    });

    let user = support::create_auth_user(role, &*connection);

    let response: HttpResponse = venues::add_to_organization((state, path, json, user)).into();

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let organization_venue: OrganizationVenue = serde_json::from_str(&body).unwrap();
    assert_eq!(organization_venue.organization_id, organization.id);
    assert_eq!(organization_venue.venue_id, venue.id);
}

pub fn add_to_organization_where_link_already_exists(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    //create user
    let user = User::create(
        "Jeff",
        "Wilco",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let user = user.add_role(Roles::Admin, &*connection).unwrap();

    //create organization
    let organization = Organization::create(user.id, &"testOrganization")
        .commit(&*connection)
        .unwrap();
    //create venue
    let venue = Venue::create("NewVenue").commit(&*connection).unwrap();
    venue
        .add_to_organization(&organization.id, &*connection)
        .unwrap();

    //link venues to organization
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let json = Json(AddVenueToOrganizationRequest {
        organization_id: organization.id,
    });

    let user = support::create_auth_user(role, &*connection);
    let response: HttpResponse = venues::add_to_organization((state, path, json, user)).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }

    assert_eq!(response.status(), StatusCode::CONFLICT);
}
