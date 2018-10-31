use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path, Query};
use bigneon_api::controllers::comps::{self, NewCompRequest};
use bigneon_api::models::{CompPathParameters, PathParameters};
use bigneon_db::models::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn index(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let user = database.create_user().finish();
    let hold = database
        .create_hold()
        .with_hold_type(HoldTypes::Comp)
        .finish();
    let event = Event::find(hold.event_id, &connection).unwrap();
    let organization = event.organization(&connection).unwrap();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let comp1 = database
        .create_comp()
        .with_hold(&hold)
        .with_name("Comp1".into())
        .finish();
    let comp2 = database
        .create_comp()
        .with_hold(&hold)
        .with_name("Comp2".into())
        .finish();
    let expected_comps = vec![comp1, comp2];

    let test_request = TestRequest::create_with_uri(&format!("/limits?"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = hold.id;

    let response: HttpResponse = comps::index((
        database.connection.into(),
        path,
        query_parameters,
        auth_user,
    )).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    let counter = expected_comps.len() as u32;
    let wrapped_expected_orgs = Payload {
        data: expected_comps,
        paging: Paging {
            page: 0,
            limit: counter,
            sort: "".to_string(),
            dir: SortingDir::Asc,
            total: counter,
            tags: Vec::new(),
        },
    };

    let expected_json = serde_json::to_string(&wrapped_expected_orgs).unwrap();
    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(body, expected_json);
    } else {
        support::expects_unauthorized(
            &response,
            Some("User does not have the required permissions"),
        );
    }
}

pub fn show(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let user = database.create_user().finish();
    let hold = database
        .create_hold()
        .with_hold_type(HoldTypes::Comp)
        .finish();
    let event = Event::find(hold.event_id, &connection).unwrap();
    let organization = event.organization(&connection).unwrap();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let comp = database.create_comp().with_hold(&hold).finish();
    let expected_json = serde_json::to_string(&comp).unwrap();

    let test_request = TestRequest::create_with_uri_custom_params("/", vec!["hold_id", "comp_id"]);
    let mut path = Path::<CompPathParameters>::extract(&test_request.request).unwrap();
    path.hold_id = hold.id;
    path.comp_id = comp.id;

    let response: HttpResponse = comps::show((database.connection.into(), path, auth_user)).into();

    if should_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let body = support::unwrap_body_to_string(&response).unwrap();
        assert_eq!(body, expected_json);
    } else {
        support::expects_unauthorized(
            &response,
            Some("User does not have the required permissions"),
        );
    }
}

pub fn destroy(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let user = database.create_user().finish();
    let hold = database
        .create_hold()
        .with_hold_type(HoldTypes::Comp)
        .finish();
    let event = Event::find(hold.event_id, &connection).unwrap();
    let organization = event.organization(&connection).unwrap();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let comp = database.create_comp().with_hold(&hold).finish();

    let test_request = TestRequest::create_with_uri_custom_params("/", vec!["hold_id", "comp_id"]);
    let mut path = Path::<CompPathParameters>::extract(&test_request.request).unwrap();
    path.hold_id = hold.id;
    path.comp_id = comp.id;

    let response: HttpResponse =
        comps::destroy((database.connection.into(), path, auth_user)).into();

    if should_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let comp = Comp::find(hold.id, comp.id, &connection);
        assert!(comp.is_err());
    } else {
        support::expects_unauthorized(
            &response,
            Some("User does not have the required permissions"),
        );
    }
}

pub fn create(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let user = database.create_user().finish();
    let hold = database
        .create_hold()
        .with_hold_type(HoldTypes::Comp)
        .finish();
    let event = Event::find(hold.event_id, &connection).unwrap();
    let organization = event.organization(&connection).unwrap();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let name = "Comp Example".to_string();
    let email = Some("email@address.com".to_string());
    let quantity = 10;

    let json = Json(NewCompRequest {
        name: name.clone(),
        email: email.clone(),
        phone: None,
        quantity: quantity,
    });

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = hold.id;

    let response: HttpResponse =
        comps::create((database.connection.into(), json, path, auth_user)).into();

    if should_test_succeed {
        let body = support::unwrap_body_to_string(&response).unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let comp: Comp = serde_json::from_str(&body).unwrap();
        assert_eq!(comp.name, name);
        assert_eq!(comp.hold_id, hold.id);
        assert_eq!(comp.email, email);
        assert_eq!(comp.quantity, 10);
    } else {
        support::expects_unauthorized(
            &response,
            Some("User does not have the required permissions"),
        );
    }
}

pub fn update(role: Roles, should_test_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let user = database.create_user().finish();
    let comp = database.create_comp().finish();
    let hold = Hold::find(comp.hold_id, &connection).unwrap();
    let event = Event::find(hold.event_id, &connection).unwrap();
    let organization = event.organization(&connection).unwrap();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);

    let name = "New Name";
    let test_request = TestRequest::create_with_uri_custom_params("/", vec!["hold_id", "comp_id"]);
    let mut path = Path::<CompPathParameters>::extract(&test_request.request).unwrap();
    path.hold_id = hold.id;
    path.comp_id = comp.id;

    let json = Json(UpdateCompAttributes {
        name: Some(name.into()),
        ..Default::default()
    });

    let response: HttpResponse =
        comps::update((database.connection.into(), json, path, auth_user)).into();
    let body = support::unwrap_body_to_string(&response).unwrap();

    if should_test_succeed {
        assert_eq!(response.status(), StatusCode::OK);
        let updated_comp: Comp = serde_json::from_str(&body).unwrap();
        assert_eq!(updated_comp.name, name);
    } else {
        support::expects_unauthorized(
            &response,
            Some("User does not have the required permissions"),
        );
    }
}
