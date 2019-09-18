use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path, Query};
use bigneon_api::controllers::settlements;
use bigneon_api::models::PathParameters;
use bigneon_db::prelude::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

pub fn index(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let organization = database.create_organization().finish();
    let organization2 = database.create_organization().finish();
    let user = database.create_user().finish();

    let settlement = database
        .create_settlement()
        .with_organization(&organization)
        .finish();
    let _settlement2 = database
        .create_settlement()
        .with_organization(&organization2)
        .finish();

    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let query_parameters = Query::<PagingParameters>::extract(&test_request.request).unwrap();

    let response = settlements::index((
        database.connection.clone().into(),
        query_parameters,
        path,
        auth_user,
    ));

    if should_succeed {
        let response = response.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            vec![settlement.id],
            response
                .payload()
                .data
                .iter()
                .map(|i| i.id)
                .collect::<Vec<Uuid>>()
        );
    } else {
        assert_eq!(
            response.err().unwrap().to_string(),
            "User does not have the required permissions"
        );
    }
}

pub fn show(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();

    let settlement = database
        .create_settlement()
        .with_organization(&organization)
        .finish();
    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = settlement.id;
    let response: HttpResponse =
        settlements::show((database.connection.clone().into(), path, auth_user)).into();
    if !should_succeed {
        support::expects_unauthorized(&response);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let returned_settlement: DisplaySettlement = serde_json::from_str(&body).unwrap();
    assert_eq!(
        returned_settlement,
        settlement.for_display(connection).unwrap()
    );
}

pub fn destroy(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let settlement = database
        .create_settlement()
        .with_organization(&organization)
        .finish();

    let auth_user =
        support::create_auth_user_from_user(&user, role, Some(&organization), &database);
    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = settlement.id;
    let response: HttpResponse =
        settlements::destroy((database.connection.clone().into(), path, auth_user)).into();
    if !should_succeed {
        support::expects_unauthorized(&response);
        return;
    }
    assert_eq!(response.status(), StatusCode::OK);
    assert!(Settlement::find(settlement.id, connection).is_err());
}
