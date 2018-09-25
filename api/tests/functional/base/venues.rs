use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::venues;
use bigneon_api::models::AddVenueToOrganizationRequest;
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn index(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let venue = database
        .create_venue()
        .with_name("Venue1".to_string())
        .finish();
    let venue2 = database
        .create_venue()
        .with_name("Venue2".to_string())
        .finish();

    let expected_venues = vec![venue, venue2];
    let venue_expected_json = serde_json::to_string(&expected_venues).unwrap();

    let user = support::create_auth_user(role, &database);
    let response: HttpResponse = venues::index((database.connection.into(), Some(user))).into();

    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(body, venue_expected_json);
}

pub fn create(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let name = "Venue Example";
    let region = database.create_region().finish();

    let user = support::create_auth_user(role, &database);
    let json = Json(NewVenue {
        name: name.clone().to_string(),
        region_id: Some(region.id.clone()),
        ..Default::default()
    });

    let response: HttpResponse = venues::create((database.connection.into(), json, user)).into();

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

pub fn create_with_organization(role: Roles, should_succeed: bool, same_organization: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let name = "Venue Example";
    let json = Json(NewVenue {
        name: name.clone().to_string(),
        organization_id: Some(organization.id.clone()),
        ..Default::default()
    });

    let response: HttpResponse =
        venues::create((database.connection.into(), json, auth_user.clone())).into();

    if !should_succeed {
        support::expects_unauthorized(&response);
        return;
    }
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(venue.name, name);
    assert_eq!(venue.organization_id, Some(organization.id));
    assert!(venue.is_private);
}

pub fn update(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let venue = database.create_venue().finish();
    let new_name = "New Name";

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let mut attributes: VenueEditableAttributes = Default::default();
    attributes.name = Some(new_name.to_string());
    let json = Json(attributes);

    let response: HttpResponse =
        venues::update((database.connection.into(), path, json, user)).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_venue.name, new_name);
}

pub fn toggle_privacy(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let venue = database.create_venue().finish();

    let auth_user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let response: HttpResponse =
        venues::toggle_privacy((database.connection.into(), path, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_venue: Venue = serde_json::from_str(&body).unwrap();
        assert_ne!(updated_venue.is_private, venue.is_private)
    } else {
        support::expects_unauthorized(&response);
    }
}

pub fn update_with_organization(
    role: Roles,
    should_succeed: bool,
    same_organization: bool,
    is_private: bool,
) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let organization = if same_organization && role != Roles::User {
        database.create_organization_with_user(&user, role == Roles::OrgOwner)
    } else {
        database.create_organization()
    }.finish();

    let mut venue = database
        .create_venue()
        .with_organization(&organization)
        .finish();

    if is_private {
        venue = venue.set_privacy(true, &database.connection).unwrap();
    }
    let new_name = "New Name";

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let mut attributes: VenueEditableAttributes = Default::default();
    attributes.name = Some(new_name.to_string());
    let json = Json(attributes);

    let response: HttpResponse =
        venues::update((database.connection.into(), path, json, auth_user.clone())).into();
    if !should_succeed {
        support::expects_unauthorized(&response);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let updated_venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(updated_venue.name, new_name);
}

pub fn show_from_organizations(role: Option<Roles>, should_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().with_user(&user).finish();
    let venue = database
        .create_venue()
        .with_name("Venue 1".to_string())
        .finish();
    let venue2 = database
        .create_venue()
        .with_name("Venue 2".to_string())
        .finish();
    let venue = venue
        .add_to_organization(&organization.id, &database.connection)
        .unwrap();
    let venue2 = venue2
        .add_to_organization(&organization.id, &database.connection)
        .unwrap();

    let all_venues = vec![venue, venue2];
    let venue_expected_json = serde_json::to_string(&all_venues).unwrap();

    let test_request = TestRequest::create();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;

    let mut user = None;

    if role.is_some() {
        user = Some(support::create_auth_user(role.unwrap(), &database));
    }

    let response: HttpResponse =
        venues::show_from_organizations((database.connection.into(), path, user)).into();

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
    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let json = Json(AddVenueToOrganizationRequest {
        organization_id: organization.id,
    });

    let response: HttpResponse =
        venues::add_to_organization((database.connection.into(), path, json, auth_user)).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let new_venue: Venue = serde_json::from_str(&body).unwrap();
    assert_eq!(new_venue.organization_id.unwrap(), organization.id);
}

pub fn add_to_organization_where_link_already_exists(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().with_user(&user).finish();
    let venue = database.create_venue().finish();
    let venue = venue
        .add_to_organization(&organization.id, &database.connection)
        .unwrap();

    let auth_user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = venue.id;

    let json = Json(AddVenueToOrganizationRequest {
        organization_id: organization.id,
    });

    let response: HttpResponse =
        venues::add_to_organization((database.connection.into(), path, json, auth_user)).into();
    if !should_succeed {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        return;
    }

    assert_eq!(response.status(), StatusCode::CONFLICT);
}
