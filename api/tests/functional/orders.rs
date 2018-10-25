use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path, Query};
use bigneon_api::controllers::orders;
use bigneon_api::models::{PagingParameters, PathParameters, Payload};
use bigneon_db::models::{DisplayOrder, OrderStatus, Roles};
use bigneon_db::schema;
use chrono::prelude::*;
use diesel;
use diesel::prelude::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[test]
pub fn show() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let mut order = database.create_order().for_user(&user).finish();
    let total = order.calculate_total(&database.connection).unwrap();
    order
        .add_external_payment("test".to_string(), user.id, total, &database.connection)
        .unwrap();
    assert_eq!(order.status, OrderStatus::Paid.to_string());

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = order.id;

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response: HttpResponse = orders::show((database.connection.into(), path, auth_user)).into();
    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let found_order: DisplayOrder = serde_json::from_str(&body).unwrap();
    assert_eq!(found_order.id, order.id);
}

#[test]
pub fn show_for_draft_returns_forbidden() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let order = database.create_order().for_user(&user).finish();
    assert_eq!(order.status, OrderStatus::Draft.to_string());

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = order.id;

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let response: HttpResponse = orders::show((database.connection.into(), path, auth_user)).into();
    support::expects_forbidden(&response, Some("You do not have access to this order"));
}

#[test]
pub fn index() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let mut order1 = database.create_order().for_user(&user).finish();
    let date1 = NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 10, 11);
    let date2 = NaiveDate::from_ymd(2017, 7, 9).and_hms(9, 10, 11);
    let total = order1.calculate_total(&database.connection).unwrap();
    order1
        .add_external_payment("test".to_string(), user.id, total, &database.connection)
        .unwrap();
    order1 = diesel::update(&order1)
        .set(schema::orders::order_date.eq(date1))
        .get_result(&*database.connection)
        .unwrap();
    let mut order2 = database.create_order().for_user(&user).finish();
    let total = order2.calculate_total(&database.connection).unwrap();
    order2
        .add_external_payment(
            "test".to_string(),
            user.id,
            total - 100,
            &database.connection,
        ).unwrap();
    order2 = diesel::update(&order2)
        .set(schema::orders::order_date.eq(date2))
        .get_result(&*database.connection)
        .unwrap();

    assert_eq!(order1.status, OrderStatus::Paid.to_string());
    assert_eq!(order2.status, OrderStatus::PartiallyPaid.to_string());

    let auth_user = support::create_auth_user_from_user(&user, Roles::User, None, &database);
    let test_request = TestRequest::create_with_uri(&format!("/?"));
    let query_parameters =
        Query::<PagingParameters>::from_request(&test_request.request, &()).unwrap();
    let response: HttpResponse = orders::index((
        database.connection.clone().into(),
        query_parameters,
        auth_user,
    )).into();

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();

    let orders: Payload<DisplayOrder> = serde_json::from_str(body).unwrap();
    assert_eq!(orders.data.len(), 2);
    let order_ids: Vec<Uuid> = orders.data.iter().map(|o| o.id).collect();
    assert_eq!(order_ids, vec![order2.id, order1.id]);
}
