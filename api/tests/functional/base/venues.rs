use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::venues::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_api::models::AddVenueToOrganizationRequest;
use bigneon_db::models::{NewVenue, OrganizationVenue, Roles, Venue, VenueEditableAttributes};
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

pub fn index(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let venue = database.create_venue().finish();
    let venue2 = database.create_venue().finish();

    let expected_venues = vec![venue, venue2];
    let venue_expected_json = serde_json::to_string(&expected_venues).unwrap();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let response: HttpResponse = venues::index((state, user)).into();

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
    let venue = database.create_venue().finish();
    let venue_expected_json = serde_json::to_string(&venue).unwrap();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let response: HttpResponse = venues::show((state, path, user)).into();
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
    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = Uuid::new_v4();

    let response: HttpResponse = venues::show((state, path, user)).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

pub fn create(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let name = "Venue Example";
    let region = database.create_region().finish();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let json = Json(NewVenue {
        name: name.clone().to_string(),
        region_id: Some(region.id.clone()),
        address: None,
        country: None,
        city: None,
        phone: None,
        state: None,
        postal_code: None,
    });

    let response: HttpResponse = venues::create((state, json, user)).into();

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(venue.name, name);
    assert_eq!(venue.region_id, Some(region.id));
}

pub fn update(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let venue = database.create_venue().finish();
    let new_name = "New Name";

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let mut attributes: VenueEditableAttributes = Default::default();
    attributes.name = Some(new_name.to_string());
    let json = Json(attributes);

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
    let user = database.create_user().finish();
    let organization = database.create_organization().with_user(&user).finish();
    let venue = database.create_venue().finish();
    let venue2 = database.create_venue().finish();
    venue
        .add_to_organization(&organization.id, &*database.get_connection())
        .unwrap();
    venue2
        .add_to_organization(&organization.id, &*database.get_connection())
        .unwrap();

    let all_venues = vec![venue, venue2];
    let venue_expected_json = serde_json::to_string(&all_venues).unwrap();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let response: HttpResponse = venues::show_from_organizations((state, path, user)).into();

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
    let user = database.create_user().finish();
    let organization = database.create_organization().with_user(&user).finish();
    let venue = database.create_venue().finish();
    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let json = Json(AddVenueToOrganizationRequest {
        organization_id: organization.id,
    });

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
    let user = database.create_user().finish();
    let organization = database.create_organization().with_user(&user).finish();
    let venue = database.create_venue().finish();
    venue
        .add_to_organization(&organization.id, &*database.get_connection())
        .unwrap();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let json = Json(AddVenueToOrganizationRequest {
        organization_id: organization.id,
    });

    let response: HttpResponse = venues::add_to_organization((state, path, json, user)).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }

    assert_eq!(response.status(), StatusCode::CONFLICT);
}
