use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::holds::{self, *};
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use functional::base;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::holds::create(Roles::OrgMember, true);
    }
    #[test]
    fn create_admin() {
        base::holds::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        base::holds::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        base::holds::create(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        base::holds::update(Roles::OrgMember, true);
    }
    #[test]
    fn update_admin() {
        base::holds::update(Roles::Admin, true);
    }
    #[test]
    fn update_user() {
        base::holds::update(Roles::User, false);
    }
    #[test]
    fn update_org_owner() {
        base::holds::update(Roles::OrgOwner, true);
    }
}

#[test]
fn create_with_validation_errors() {
    let database = TestDatabase::new();
    let organization = database.create_organization().finish();
    let user = database.create_user().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, Roles::OrgOwner, Some(&organization), &database);
    let event = database
        .create_event()
        .with_tickets()
        .with_organization(&organization)
        .finish();

    let name = "Hold Example".to_string();
    let redemption_code = "IHAVEACODE".to_string();
    let hold_type = HoldTypes::Discount;

    let json = Json(CreateHoldRequest {
        name: name.clone(),
        redemption_code: redemption_code,
        discount_in_cents: None,
        hold_type,
        end_at: None,
        max_per_order: None,
        quantity: 2,
        ticket_type_id: event.ticket_types(&database.connection.clone()).unwrap()[0].id,
    });

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse =
        holds::create((database.connection.into(), json, path, auth_user)).into();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.error().is_some());

    let validation_response = support::validation_response_from_response(&response).unwrap();
    let discount_in_cents = validation_response.fields.get("discount_in_cents").unwrap();
    assert_eq!(discount_in_cents[0].code, "required");
    assert_eq!(
        &discount_in_cents[0].message.clone().unwrap().into_owned(),
        "Discount required for hold type Discount"
    );
}

#[test]
fn update_with_validation_errors() {
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
        support::create_auth_user_from_user(&user, Roles::OrgOwner, Some(&organization), &database);
    let name = "New Name";

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = hold.id;

    let json = Json(UpdateHoldRequest {
        name: Some(name.into()),
        hold_type: Some(HoldTypes::Discount),
        ..Default::default()
    });

    let response: HttpResponse =
        holds::update((database.connection.into(), json, path, auth_user)).into();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.error().is_some());

    let validation_response = support::validation_response_from_response(&response).unwrap();
    let discount_in_cents = validation_response.fields.get("discount_in_cents").unwrap();
    assert_eq!(discount_in_cents[0].code, "required");
    assert_eq!(
        &discount_in_cents[0].message.clone().unwrap().into_owned(),
        "Discount required for hold type Discount"
    );
}

#[test]
pub fn add_remove_from_hold_with_validation_errors() {
    let database = TestDatabase::new();
    let connection = database.connection.clone();
    let user = database.create_user().finish();
    let hold = database
        .create_hold()
        .with_hold_type(HoldTypes::Comp)
        .finish();
    // 4 comps exist for this hold so setting the quantity < 4 will trigger validation error
    database
        .create_comp()
        .with_hold(&hold)
        .with_quantity(4)
        .finish();
    let event = Event::find(hold.event_id, &connection).unwrap();
    let organization = event.organization(&connection).unwrap();
    let auth_user =
        support::create_auth_user_from_user(&user, Roles::OrgOwner, Some(&organization), &database);
    assert_eq!(hold.quantity(&connection).unwrap(), (10, 10));

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = hold.id;

    let json = Json(UpdateHoldRequest {
        quantity: Some(3),
        ..Default::default()
    });
    let response: HttpResponse =
        holds::update((database.connection.into(), json, path, auth_user)).into();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.error().is_some());

    let validation_response = support::validation_response_from_response(&response).unwrap();
    let quantity = validation_response.fields.get("quantity").unwrap();
    assert_eq!(
        quantity[0].code,
        "assigned_comp_count_greater_than_quantity"
    );
    assert_eq!(
        &quantity[0].message.clone().unwrap().into_owned(),
        "Existing comp total quantity greater than new quantity"
    );
}

#[test]
pub fn read_hold() {
    let database = TestDatabase::new();
    let organization = database.create_organization().finish();
    let user = database.create_user().finish();
    let auth_user =
        support::create_auth_user_from_user(&user, Roles::OrgOwner, Some(&organization), &database);
    let event = database
        .create_event()
        .with_tickets()
        .with_organization(&organization)
        .finish();

    let name = "Hold Example".to_string();
    let redemption_code = "IHAVEACODE".to_string();
    let hold_type = HoldTypes::Discount;

    let json = Json(CreateHoldRequest {
        name: name.clone(),
        redemption_code: redemption_code,
        discount_in_cents: Some(100),
        hold_type,
        end_at: None,
        max_per_order: None,
        quantity: 2,
        ticket_type_id: event.ticket_types(&database.connection.clone()).unwrap()[0].id,
    });

    let test_request = TestRequest::create();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;

    let response: HttpResponse = holds::create((
        database.connection.clone().into(),
        json,
        path,
        auth_user.clone(),
    )).into();
    let body = support::unwrap_body_to_string(&response).unwrap();
    let created_hold: Hold = serde_json::from_str(body).unwrap();

    let mut hold_path = Path::<PathParameters>::extract(&test_request.request).unwrap();

    hold_path.id = created_hold.id;
    let show_response = holds::show((database.connection.into(), hold_path, auth_user)).into();
    let show_body = support::unwrap_body_to_string(&show_response).unwrap();

    #[derive(Deserialize)]
    struct R {
        id: Uuid,
    }
    let fetched_hold: R = serde_json::from_str(show_body).unwrap();

    assert_eq!(created_hold.id, fetched_hold.id);
}
