use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::events::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_db::models::{
    Event, EventEditableAttributes, EventInterest, NewEvent, Organization, Roles, User, Venue,
};
use chrono::prelude::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    //create prerequisites
    let user = User::create(
        "Jeff",
        "Roen",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*connection)
        .unwrap();
    let venue = Venue::create(&"Venue").commit(&*connection).unwrap();
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

    let user = support::create_auth_user(role, &*connection);

    let response = events::create((state, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let event_expected_json = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, event_expected_json);
    }
}

pub fn update(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    //create prerequisites
    let user = User::create(
        "Jeff",
        "Roen",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*connection)
        .unwrap();
    let venue = Venue::create(&"Venue").commit(&*connection).unwrap();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*connection)
        .unwrap();

    let new_name = "New Event Name";
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let json = Json(EventEditableAttributes {
        name: Some(new_name.clone().to_string()),
        organization_id: Some(event.organization_id.clone()),
        venue_id: Some(event.venue_id.clone()),
        event_start: Some(event.event_start.clone()),
        ticket_sell_date: Some(event.ticket_sell_date.clone()),
    });
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let user = support::create_auth_user(role, &*connection);

    let response = events::update((state, path, json, user));

    let body = support::unwrap_body_to_string(&response).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_event: Event = serde_json::from_str(&body).unwrap();
        assert_eq!(updated_event.name, new_name);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let updated_event = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, updated_event);
    }
}

pub fn add_interest(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    //create prerequisites
    let user = User::create(
        "Jeff",
        "Roen",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*connection)
        .unwrap();
    let venue = Venue::create(&"Venue").commit(&*connection).unwrap();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*connection)
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let user = support::create_auth_user(role, &*connection);

    let response = events::add_interest((state, path, user));

    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::CREATED);
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let updated_event = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, updated_event);
    }
}

pub fn remove_interest(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.get_connection();
    //create prerequisites
    let user = User::create(
        "Jeff",
        "Roen",
        "jeff@tari.com",
        "555-555-5555",
        "examplePassword",
    ).commit(&*connection)
        .unwrap();
    let organization = Organization::create(user.id, "Organization")
        .commit(&*connection)
        .unwrap();
    let venue = Venue::create(&"Venue").commit(&*connection).unwrap();
    let event = Event::create(
        "NewEvent",
        organization.id,
        venue.id,
        NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    ).commit(&*connection)
        .unwrap();
    let event_like_response = EventInterest::create(event.id, user.id)
        .commit(&*connection)
        .unwrap();

    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let user = support::create_auth_user_from_user(&user, role, &*connection);

    let response = events::remove_interest((state, path, user));

    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, "1");
    } else {
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let temp_json = HttpResponse::Unauthorized().json(json!({"error": "Unauthorized"}));
        let updated_event = support::unwrap_body_to_string(&temp_json).unwrap();
        assert_eq!(body, updated_event);
    }
}
