use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use chrono::prelude::*;

#[test]
fn box_office_sales_summary_report() {
    let project = TestProject::new();
    let connection = project.get_connection();
    let box_office_user = project
        .create_user()
        .with_first_name("BoxOfficeUser1")
        .finish();
    let box_office_user2 = project
        .create_user()
        .with_first_name("BoxOfficeUser2")
        .finish();
    let creator = project.create_user().finish();
    let user = project.create_user().finish();
    let user2 = project.create_user().finish();
    let user3 = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_member(&box_office_user, Roles::OrgBoxOffice)
        .with_member(&box_office_user2, Roles::OrgBoxOffice)
        .with_fee_schedule(&project.create_fee_schedule().finish(creator.id))
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_event_start(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))
        .with_name("Event1".to_string())
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let event2 = project
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
        project
            .create_order()
            .quantity(2)
            .for_event(&event)
            .for_user(&box_office_user)
            .on_behalf_of_user(&user)
            .is_paid()
            .with_external_payment_type(ExternalPaymentType::Voucher)
            .finish(),
    );
    box_office_user_orders.push(
        project
            .create_order()
            .quantity(1)
            .for_event(&event2)
            .for_user(&box_office_user)
            .on_behalf_of_user(&user)
            .is_paid()
            .with_external_payment_type(ExternalPaymentType::CreditCard)
            .finish(),
    );
    box_office_user_orders.push(
        project
            .create_order()
            .quantity(1)
            .for_event(&event2)
            .for_user(&box_office_user)
            .on_behalf_of_user(&user3)
            .is_paid()
            .with_external_payment_type(ExternalPaymentType::Cash)
            .finish(),
    );
    box_office_user2_orders.push(
        project
            .create_order()
            .quantity(2)
            .for_event(&event)
            .for_user(&box_office_user2)
            .on_behalf_of_user(&user2)
            .is_paid()
            .with_external_payment_type(ExternalPaymentType::Cash)
            .finish(),
    );

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
                        quantity: 1,
                        total_sales_in_cents: 150,
                    },
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Credit Card".to_string(),
                        quantity: 1,
                        total_sales_in_cents: 150,
                    },
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Voucher".to_string(),
                        quantity: 2,
                        total_sales_in_cents: 300,
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
                        quantity: 2,
                        total_sales_in_cents: 300,
                    },
                    BoxOfficeSalesSummaryPaymentRow {
                        payment_type: "Credit Card".to_string(),
                        quantity: 0,
                        total_sales_in_cents: 0,
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
                quantity: 3,
                total_sales_in_cents: 450,
            },
            BoxOfficeSalesSummaryPaymentRow {
                payment_type: "Credit Card".to_string(),
                quantity: 1,
                total_sales_in_cents: 150,
            },
            BoxOfficeSalesSummaryPaymentRow {
                payment_type: "Voucher".to_string(),
                quantity: 2,
                total_sales_in_cents: 300,
            },
        ],
    };

    let report_data =
        Report::box_office_sales_summary_report(organization.id, None, None, connection).unwrap();
    assert_eq!(expected_report_data, report_data);
}
