use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::auth::user::User as AuthUser;
use bigneon_api::controllers::events::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{Event, NewEvent, Organization, Roles, User, Venue};
use chrono::prelude::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

pub fn index(role: Roles) {
    let database = TestDatabase::new();
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*database.get_connection())
        .unwrap();
    let venue = Venue::create(&"Venue")
        .commit(&*database.get_connection())
        .unwrap();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*database.get_connection())
        .unwrap();
    let event2 = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2015, 7, 8).and_hms(9, 10, 11),
    ).commit(&*database.get_connection())
        .unwrap();

    let expected_events = vec![event, event2];
    let events_expected_json = serde_json::to_string(&expected_events).unwrap();
    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let should_test_true = user.is_in_role(Roles::Guest);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let response = events::index((state, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, events_expected_json);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let events_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, events_expected_json);
    }
}

pub fn show(role: Roles) {
    let database = TestDatabase::new();
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*database.get_connection())
        .unwrap();
    let venue = Venue::create(&"Venue")
        .commit(&*database.get_connection())
        .unwrap();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*database.get_connection())
        .unwrap();
    let event_expected_json = serde_json::to_string(&event).unwrap();

    let test_request = TestRequest::create_with_route(
        database,
        &"/events/{id}",
        &format!("/events/{}", event.id.to_string()),
    );
    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let should_test_true = user.is_in_role(Roles::Guest);
    let state = test_request.extract_state();
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();

    let response = events::show((state, path, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, event_expected_json);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let event_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, event_expected_json);
    }
}

pub fn create(role: Roles) {
    let database = TestDatabase::new();
    //create prerequisites
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*database.get_connection())
        .unwrap();
    let venue = Venue::create(&"Venue")
        .commit(&*database.get_connection())
        .unwrap();
    //create event
    let name = "event Example";
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let new_event = NewEvent {
        name: name.clone().to_string(),
        organization_id: organization.id,
        venue_id: venue.id,
        event_start: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    };
    let json = Json(new_event);
    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let should_test_true = user.is_in_role(Roles::OrgOwner);
    let response = events::create((state, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let event_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, event_expected_json);
    }
}

pub fn update(role: Roles) {
    let database = TestDatabase::new();
    //create prerequisites
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*database.get_connection())
        .unwrap();
    let venue = Venue::create(&"Venue")
        .commit(&*database.get_connection())
        .unwrap();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*database.get_connection())
        .unwrap();

    let new_name = "New Event Name";
    let test_request = TestRequest::create_with_route(
        database,
        &"/events/{id}",
        &format!("/events/{}", event.id.to_string()),
    );
    let state = test_request.extract_state();
    let update_event = Event {
        id: event.id.clone(),
        name: new_name.clone().to_string(),
        organization_id: event.organization_id.clone(),
        venue_id: event.venue_id.clone(),
        event_start: event.event_start.clone(),
        created_at: event.created_at.clone(),
        ticket_sell_date: event.ticket_sell_date.clone(),
    };
    let path = Path::<PathParameters>::extract(&test_request.request).unwrap();

    let updated_event = serde_json::to_string(&update_event).unwrap();
    let json = Json(update_event);

    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let should_test_true = user.is_in_role(Roles::OrgOwner);
    let response = events::update((state, path, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, updated_event);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let updated_event = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, updated_event);
    }
}

pub fn show_from_organizations(role: Roles) {
    let database = TestDatabase::new();
    //create prerequisites
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*database.get_connection())
        .unwrap();
    let venue = Venue::create(&"Venue")
        .commit(&*database.get_connection())
        .unwrap();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*database.get_connection())
        .unwrap();
    let event2 = Event::create(
        "NewEvent2",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*database.get_connection())
        .unwrap();

    let all_events = vec![event, event2];
    let event_expected_json = serde_json::to_string(&all_events).unwrap();
    //find venue from organization
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let should_test_true = user.is_in_role(Roles::Guest);
    let json = Json(organization.id);
    let response = events::show_from_organizations((state, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, event_expected_json);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let event_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, event_expected_json);
    }
}

pub fn show_from_venues(role: Roles) {
    let database = TestDatabase::new();
    //create prerequisites
    let user = User::create("Jeff", "jeff@tari.com", "555-555-5555", "examplePassword")
        .commit(&*database.get_connection())
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*database.get_connection())
        .unwrap();
    let venue = Venue::create(&"Venue")
        .commit(&*database.get_connection())
        .unwrap();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*database.get_connection())
        .unwrap();
    let event2 = Event::create(
        "NewEvent2",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*database.get_connection())
        .unwrap();
    //find venue from organization

    let all_events = vec![event, event2];
    let event_expected_json = serde_json::to_string(&all_events).unwrap();
    //find venue from organization
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let user = AuthUser::new(Uuid::new_v4(), vec![role]);
    let should_test_true = user.is_in_role(Roles::Guest);
    let json = Json(venue.id);
    let response = events::show_from_venues((state, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_true {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, event_expected_json);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let event_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, event_expected_json);
    }
}
