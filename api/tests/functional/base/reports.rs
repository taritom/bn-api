use actix_web::{http::StatusCode, FromRequest, HttpResponse, Path, Query};
use bigneon_api::controllers::reports::{self, *};
use bigneon_api::models::PathParameters;
use bigneon_db::models::*;
use chrono::prelude::*;
use serde_json;
use support;
use support::database::TestDatabase;
use support::test_request::TestRequest;

pub fn box_office_sales_summary(role: Roles, should_succeed: bool) {
    let database = TestDatabase::new();
    let box_office_user = database
        .create_user()
        .with_first_name("BoxOfficeUser1")
        .finish();
    let box_office_user2 = database
        .create_user()
        .with_first_name("BoxOfficeUser2")
        .finish();
    let creator = database.create_user().finish();
    let user = database.create_user().finish();
    let user2 = database.create_user().finish();
    let user3 = database.create_user().finish();
    let organization = database
        .create_organization()
        .with_member(&box_office_user, Roles::OrgBoxOffice)
        .with_member(&box_office_user2, Roles::OrgBoxOffice)
        .with_fee_schedule(&database.create_fee_schedule().finish(creator.id))
        .finish();
    let event = database
        .create_event()
        .with_organization(&organization)
        .with_event_start(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))
        .with_name("Event1".to_string())
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let event2 = database
        .create_event()
        .with_organization(&organization)
        .with_event_start(NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 10, 11))
        .with_name("Event2".to_string())
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let mut box_office_user_orders = Vec::new();
    let mut box_office_user2_orders = Vec::new();
    box_office_user_orders.push(
        database
            .create_order()
            .quantity(2)
            .for_event(&event)
            .for_user(&box_office_user)
            .on_behalf_of_user(&user)
            .is_paid()
            .finish(),
    );
    box_office_user_orders.push(
        database
            .create_order()
            .quantity(1)
            .for_event(&event2)
            .for_user(&box_office_user)
            .on_behalf_of_user(&user)
            .is_paid()
            .finish(),
    );
    box_office_user_orders.push(
        database
            .create_order()
            .quantity(1)
            .for_event(&event2)
            .for_user(&box_office_user)
            .on_behalf_of_user(&user3)
            .is_paid()
            .finish(),
    );
    box_office_user2_orders.push(
        database
            .create_order()
            .quantity(2)
            .for_event(&event)
            .for_user(&box_office_user2)
            .on_behalf_of_user(&user2)
            .is_paid()
            .finish(),
    );

    let auth_db_user = database.create_user().finish();
    let auth_user =
        support::create_auth_user_from_user(&auth_db_user, role, Some(&organization), &database);

    let test_request = TestRequest::create_with_uri("/reports?report=box_office_sales_summary");
    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    path.id = organization.id;
    let query = Query::<ReportQueryParameters>::extract(&test_request.request).unwrap();
    let response: HttpResponse = reports::box_office_sales_summary((
        database.connection.clone().into(),
        query,
        path,
        auth_user,
    ))
    .into();

    if !should_succeed {
        support::expects_unauthorized(&response);
        return;
    }

    assert_eq!(response.status(), StatusCode::OK);
    let body = support::unwrap_body_to_string(&response).unwrap();
    let report_data: BoxOfficeSalesSummaryReport = serde_json::from_str(&body).unwrap();

    let expected_report_data = BoxOfficeSalesSummaryReport {
        operators: vec![
            BoxOfficeSalesSummaryOperatorRow {
                operator_id: box_office_user.id,
                operator_name: box_office_user.full_name(),
                events: vec![
                    BoxOfficeSalesSummaryOperatorEventRow {
                        event_name: Some("Event1".to_string()),
                        event_date: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)),
                        number_of_tickets: 2,
                        face_value_in_cents: 150,
                        total_fees_in_cents: 0,
                        total_sales_in_cents: 300,
                    },
                    BoxOfficeSalesSummaryOperatorEventRow {
                        event_name: Some("Event2".to_string()),
                        event_date: Some(NaiveDate::from_ymd(2017, 7, 8).and_hms(9, 10, 11)),
                        number_of_tickets: 2,
                        face_value_in_cents: 150,
                        total_fees_in_cents: 0,
                        total_sales_in_cents: 300,
                    },
                ],
                payments: vec![
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Cash".to_string(),
                        quantity: 0,
                        total_sales_in_cents: 0,
                    },
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Credit Card".to_string(),
                        quantity: 4,
                        total_sales_in_cents: 600,
                    },
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Voucher".to_string(),
                        quantity: 0,
                        total_sales_in_cents: 0,
                    },
                ],
            },
            BoxOfficeSalesSummaryOperatorRow {
                operator_id: box_office_user2.id,
                operator_name: box_office_user2.full_name(),
                events: vec![BoxOfficeSalesSummaryOperatorEventRow {
                    event_name: Some("Event1".to_string()),
                    event_date: Some(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11)),
                    number_of_tickets: 2,
                    face_value_in_cents: 150,
                    total_fees_in_cents: 0,
                    total_sales_in_cents: 300,
                }],
                payments: vec![
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Cash".to_string(),
                        quantity: 0,
                        total_sales_in_cents: 0,
                    },
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Credit Card".to_string(),
                        quantity: 2,
                        total_sales_in_cents: 300,
                    },
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Voucher".to_string(),
                        quantity: 0,
                        total_sales_in_cents: 0,
                    },
                ],
            },
        ],
        payments: vec![
            BoxOfficeSalesSummaryPaymentRow {
                payment_type: "Cash".to_string(),
                quantity: 0,
                total_sales_in_cents: 0,
            },
            BoxOfficeSalesSummaryPaymentRow {
                payment_type: "Credit Card".to_string(),
                quantity: 6,
                total_sales_in_cents: 900,
            },
            BoxOfficeSalesSummaryPaymentRow {
                payment_type: "Voucher".to_string(),
                quantity: 0,
                total_sales_in_cents: 0,
            },
        ],
    };

    assert_eq!(expected_report_data, report_data);
}
