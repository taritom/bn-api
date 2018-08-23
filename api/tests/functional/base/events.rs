use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::events::{self, PathParameters};
use bigneon_api::database::ConnectionGranting;
use bigneon_api::models::CreateTicketAllocationRequest;
use bigneon_db::models::{Event, EventEditableAttributes, EventInterest, NewEvent, Roles};
use chrono::prelude::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();

    let organization = database.create_organization().finish();
    let venue = database.create_venue().finish();

    let name = "event Example";
    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();
    let new_event = NewEvent {
        name: name.clone().to_string(),
        organization_id: organization.id,
        venue_id: venue.id,
        event_start: NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11),
    };
    let json = Json(new_event);

    let response: HttpResponse = events::create((state, json, user)).into();
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
    let event = database.create_event().finish();

    let new_name = "New Event Name";
    let user = support::create_auth_user(role, &database);
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

    let response: HttpResponse = events::update((state, path, json, user)).into();
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
    let event = database.create_event().finish();

    let user = support::create_auth_user(role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse = events::add_interest((state, path, user)).into();
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
    let user = database.create_user().finish();
    let event = database.create_event().finish();
    EventInterest::create(event.id, user.id)
        .commit(&*connection)
        .unwrap();

    let user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse = events::remove_interest((state, path, user)).into();
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

pub fn create_tickets(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().with_user(&user).finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();

    let user = support::create_auth_user_from_user(&user, role, &database);
    let test_request = TestRequest::create(database);
    let state = test_request.extract_state();

    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let data = CreateTicketAllocationRequest {
        name: "VIP".into(),
        tickets_delta: 100,
    };
    let response: HttpResponse = events::create_tickets((state, path, Json(data), user)).into();

    let _body = support::unwrap_body_to_string(&response).unwrap();
    if should_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        return;
    }

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
