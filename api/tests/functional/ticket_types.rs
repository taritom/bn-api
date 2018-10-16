use actix_web::{http::StatusCode, FromRequest, HttpResponse, Json, Path};
use bigneon_api::controllers::ticket_types;
use bigneon_api::controllers::ticket_types::*;
use bigneon_api::models::{EventTicketPathParameters, PathParameters};
use bigneon_db::models::*;
use chrono::prelude::*;
use functional::base;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;
use uuid::Uuid;

#[cfg(test)]
mod create_tests {
    use super::*;
    #[test]
    fn create_org_member() {
        base::ticket_types::create(Roles::OrgMember, true);
    }
    #[test]
    fn create_admin() {
        base::ticket_types::create(Roles::Admin, true);
    }
    #[test]
    fn create_user() {
        base::ticket_types::create(Roles::User, false);
    }
    #[test]
    fn create_org_owner() {
        base::ticket_types::create(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod update_tests {
    use super::*;
    #[test]
    fn update_org_member() {
        base::ticket_types::update(Roles::OrgMember, true);
    }
    #[test]
    fn update_admin() {
        base::ticket_types::update(Roles::Admin, true);
    }
    #[test]
    fn update_user() {
        base::ticket_types::update(Roles::User, false);
    }
    #[test]
    fn update_org_owner() {
        base::ticket_types::update(Roles::OrgOwner, true);
    }
}

#[cfg(test)]
mod index_tests {
    use super::*;
    #[test]
    fn index_org_member() {
        base::ticket_types::index(Roles::OrgMember, true);
    }
    #[test]
    fn index_admin() {
        base::ticket_types::index(Roles::Admin, true);
    }
    #[test]
    fn index_user() {
        base::ticket_types::index(Roles::User, false);
    }
    #[test]
    fn index_org_owner() {
        base::ticket_types::index(Roles::OrgOwner, true);
    }
}

#[test]
pub fn create_with_overlapping_periods() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::Admin, None, &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .finish();
    //Construct Ticket creation and pricing request
    let test_request = TestRequest::create();
    let state = test_request.extract_state();
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = event.id;
    let mut ticket_pricing: Vec<CreateTicketPricingRequest> = Vec::new();
    let start_date = NaiveDate::from_ymd(2018, 5, 1).and_hms(6, 20, 21);
    let middle_date = NaiveDate::from_ymd(2018, 6, 2).and_hms(7, 45, 31);
    let end_date = NaiveDate::from_ymd(2018, 7, 3).and_hms(9, 23, 23);
    ticket_pricing.push(CreateTicketPricingRequest {
        name: String::from("Early bird"),
        price_in_cents: 10000,
        start_date,
        end_date: middle_date,
    });
    ticket_pricing.push(CreateTicketPricingRequest {
        name: String::from("Base"),
        price_in_cents: 20000,
        start_date: start_date,
        end_date,
    });
    let request_data = CreateTicketTypeRequest {
        name: "VIP".into(),
        capacity: 1000,
        start_date,
        end_date,
        ticket_pricing,
        increment: None,
    };
    let response: HttpResponse = ticket_types::create((
        database.connection.into(),
        path,
        Json(request_data),
        auth_user,
        state,
    )).into();

    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.error().is_some());

    #[derive(Deserialize)]
    struct Response {
        error: String,
    }
    let deserialized_response: Response = serde_json::from_str(&body).unwrap();
    assert_eq!(deserialized_response.error, "Validation error");
}

#[test]
pub fn update_with_invalid_id() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::Admin, None, &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    //Retrieve created ticket type and pricing
    let created_ticket_type = &event.ticket_types(&database.connection).unwrap()[0];
    let created_ticket_capacity = created_ticket_type
        .ticket_capacity(&database.connection)
        .unwrap();
    created_ticket_type
        .ticket_pricing(&database.connection)
        .unwrap();

    //Construct update request
    let test_request = TestRequest::create_with_uri_event_ticket("/");
    let mut path = Path::<EventTicketPathParameters>::extract(&test_request.request).unwrap();
    path.event_id = event.id;
    path.ticket_type_id = created_ticket_type.id;

    let mut request_ticket_pricing: Vec<UpdateTicketPricingRequest> = Vec::new();
    let start_date = Some(NaiveDate::from_ymd(2018, 5, 1).and_hms(6, 20, 21));
    let end_date = Some(NaiveDate::from_ymd(2018, 7, 3).and_hms(9, 23, 23));
    request_ticket_pricing.push(UpdateTicketPricingRequest {
        id: Some(Uuid::new_v4()),
        name: Some(String::from("Base")),
        start_date,
        end_date,
        price_in_cents: Some(20000),
    });
    let request_data = UpdateTicketTypeRequest {
        name: Some("Updated VIP".into()),
        capacity: Some(created_ticket_capacity),
        start_date,
        end_date,
        ticket_pricing: Some(request_ticket_pricing),
        increment: None,
    };

    //Send update request
    let response: HttpResponse = ticket_types::update((
        database.connection.clone().into(),
        path,
        Json(request_data),
        auth_user,
    )).into();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(response.error().is_some());
}

#[test]
pub fn update_with_overlapping_periods() {
    let database = TestDatabase::new();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let auth_user = support::create_auth_user_from_user(&user, Roles::Admin, None, &database);
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    //Retrieve created ticket type and pricing
    let created_ticket_type = &event.ticket_types(&database.connection).unwrap()[0];
    let created_ticket_capacity = created_ticket_type
        .ticket_capacity(&database.connection)
        .unwrap();
    let created_ticket_pricing = created_ticket_type
        .ticket_pricing(&database.connection)
        .unwrap();

    //Construct update request
    let test_request = TestRequest::create_with_uri_event_ticket("/");
    let mut path = Path::<EventTicketPathParameters>::extract(&test_request.request).unwrap();
    path.event_id = event.id;
    path.ticket_type_id = created_ticket_type.id;

    let mut request_ticket_pricing: Vec<UpdateTicketPricingRequest> = Vec::new();
    let start_date = Some(NaiveDate::from_ymd(2018, 5, 1).and_hms(6, 20, 21));
    let middle_date = Some(NaiveDate::from_ymd(2018, 6, 2).and_hms(7, 45, 31));
    let end_date = Some(NaiveDate::from_ymd(2018, 7, 3).and_hms(9, 23, 23));
    let new_pricing_name = String::from("Online");
    //Remove 1st pricing, modify 2nd pricing and add new additional pricing
    request_ticket_pricing.push(UpdateTicketPricingRequest {
        id: Some(created_ticket_pricing[1].id),
        name: Some(String::from("Base")),
        start_date: start_date,
        end_date,
        price_in_cents: Some(20000),
    });
    request_ticket_pricing.push(UpdateTicketPricingRequest {
        id: None,
        name: Some(new_pricing_name.clone()),
        start_date,
        end_date: middle_date,
        price_in_cents: Some(15000),
    });
    let request_data = UpdateTicketTypeRequest {
        name: Some("Updated VIP".into()),
        capacity: Some(created_ticket_capacity),
        start_date,
        end_date,
        ticket_pricing: Some(request_ticket_pricing),
        increment: None,
    };

    //Send update request
    let response: HttpResponse = ticket_types::update((
        database.connection.clone().into(),
        path,
        Json(request_data),
        auth_user,
    )).into();

    let body = support::unwrap_body_to_string(&response).unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert!(response.error().is_some());

    #[derive(Deserialize)]
    struct Response {
        error: String,
    }
    let deserialized_response: Response = serde_json::from_str(&body).unwrap();
    assert_eq!(deserialized_response.error, "Validation error");
}
