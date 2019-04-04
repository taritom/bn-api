use bigneon_db::dev::TestProject;
use bigneon_db::models::*;
use bigneon_db::schema::orders;
use chrono::{Duration, NaiveDate, Utc};
use diesel;
use diesel::prelude::*;

#[test]
fn transaction_detail_report() {
    let project = TestProject::new();
    let connection = project.get_connection();

    let creator = project.create_user().finish();
    let organization = project
        .create_organization()
        .with_event_fee()
        .with_fee_schedule(&project.create_fee_schedule().finish(creator.id))
        .finish();
    let event = project
        .create_event()
        .with_organization(&organization)
        .with_name("Event1".to_string())
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type = event
        .ticket_types(true, None, connection)
        .unwrap()
        .remove(0);
    let event2 = project
        .create_event()
        .with_organization(&organization)
        .with_name("Event2".to_string())
        .with_tickets()
        .with_ticket_pricing()
        .finish();
    let ticket_type2 = event2
        .ticket_types(true, None, connection)
        .unwrap()
        .remove(0);
    let organization2 = project
        .create_organization()
        .with_fee_schedule(&project.create_fee_schedule().finish(creator.id))
        .finish();
    let event3 = project
        .create_event()
        .with_organization(&organization2)
        .with_name("Event3".to_string())
        .with_tickets()
        .with_ticket_pricing()
        .finish();

    let fee_schedule = FeeSchedule::find(organization.fee_schedule_id, connection).unwrap();
    let ticket_pricing = ticket_type
        .current_ticket_pricing(false, connection)
        .unwrap();
    let fee_schedule_range = fee_schedule
        .get_range(ticket_pricing.price_in_cents, connection)
        .unwrap();
    let ticket_pricing2 = ticket_type2
        .current_ticket_pricing(false, connection)
        .unwrap();
    let fee_schedule_range2 = fee_schedule
        .get_range(ticket_pricing2.price_in_cents, connection)
        .unwrap();

    let user = project.create_user().with_first_name("Bob".into()).finish();
    let user2 = project
        .create_user()
        .with_first_name("Bobby".into())
        .finish();
    let user3 = project
        .create_user()
        .with_first_name("Dan".into())
        .with_last_name("Bob".into())
        .finish();
    let user4 = project
        .create_user()
        .with_first_name("Dan".into())
        .with_last_name("Smith".into())
        .finish();

    let mut order = project
        .create_order()
        .quantity(2)
        .for_event(&event)
        .for_user(&user)
        .is_paid()
        .with_external_payment_type(ExternalPaymentType::Voucher)
        .finish();
    let order_paid_at = Utc::now().naive_utc() - Duration::days(5);
    order = diesel::update(orders::table.filter(orders::id.eq(order.id)))
        .set(orders::paid_at.eq(order_paid_at))
        .get_result::<Order>(connection)
        .unwrap();

    let mut order2 = project
        .create_order()
        .quantity(2)
        .for_event(&event)
        .for_user(&user2)
        .is_paid()
        .with_external_payment_type(ExternalPaymentType::Voucher)
        .finish();
    let order2_paid_at = Utc::now().naive_utc() - Duration::days(4);
    order2 = diesel::update(orders::table.filter(orders::id.eq(order2.id)))
        .set(orders::paid_at.eq(order2_paid_at))
        .get_result::<Order>(connection)
        .unwrap();

    let mut order3 = project
        .create_order()
        .quantity(2)
        .for_event(&event2)
        .for_user(&user3)
        .is_paid()
        .with_external_payment_type(ExternalPaymentType::Voucher)
        .finish();
    let order3_paid_at = Utc::now().naive_utc() - Duration::days(3);
    order3 = diesel::update(orders::table.filter(orders::id.eq(order3.id)))
        .set(orders::paid_at.eq(order3_paid_at))
        .get_result::<Order>(connection)
        .unwrap();

    let mut order4 = project
        .create_order()
        .quantity(2)
        .for_event(&event)
        .for_user(&user4)
        .is_paid()
        .with_external_payment_type(ExternalPaymentType::Voucher)
        .finish();
    let order4_paid_at = Utc::now().naive_utc() - Duration::days(2);
    order4 = diesel::update(orders::table.filter(orders::id.eq(order4.id)))
        .set(orders::paid_at.eq(order4_paid_at))
        .get_result::<Order>(connection)
        .unwrap();

    let _order5 = project
        .create_order()
        .quantity(2)
        .for_event(&event3)
        .for_user(&user)
        .is_paid()
        .with_external_payment_type(ExternalPaymentType::Voucher)
        .finish();
    let _order6 = project
        .create_order()
        .quantity(2)
        .for_event(&event3)
        .for_user(&user3)
        .is_paid()
        .with_external_payment_type(ExternalPaymentType::Voucher)
        .finish();

    // No query, for event
    let result = Report::transaction_detail_report(
        None,
        Some(event.id),
        None,
        None,
        None,
        0,
        100,
        connection,
    )
    .unwrap();
    let expected_results = vec![
        build_transaction_report_row(
            3,
            &user,
            &order,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
        build_transaction_report_row(
            3,
            &user2,
            &order2,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
        build_transaction_report_row(
            3,
            &user4,
            &order4,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
    ];
    assert_eq!(result.data, expected_results);
    assert_eq!(result.paging.total, 3);

    // No query, for organization
    let result = Report::transaction_detail_report(
        None,
        None,
        Some(organization.id),
        None,
        None,
        0,
        100,
        connection,
    )
    .unwrap();
    let expected_results = vec![
        build_transaction_report_row(
            4,
            &user,
            &order,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
        build_transaction_report_row(
            4,
            &user2,
            &order2,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
        build_transaction_report_row(
            4,
            &user3,
            &order3,
            &event2,
            &ticket_type2,
            &fee_schedule_range2,
            2,
            ticket_pricing2.price_in_cents,
        ),
        build_transaction_report_row(
            4,
            &user4,
            &order4,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
    ];
    assert_eq!(result.data, expected_results);
    assert_eq!(result.paging.total, 4);

    // With query, for organization (query finds user's name)
    let query = "Bob".to_string();
    let result = Report::transaction_detail_report(
        Some(query),
        None,
        Some(organization.id),
        None,
        None,
        0,
        100,
        connection,
    )
    .unwrap();
    let expected_results = vec![
        build_transaction_report_row(
            3,
            &user,
            &order,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
        build_transaction_report_row(
            3,
            &user2,
            &order2,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
        build_transaction_report_row(
            3,
            &user3,
            &order3,
            &event2,
            &ticket_type2,
            &fee_schedule_range2,
            2,
            ticket_pricing2.price_in_cents,
        ),
    ];
    assert_eq!(result.data, expected_results);
    assert_eq!(result.paging.total, 3);

    // With query, for organization (query finds user's email)
    let query = user.email.clone();
    let result = Report::transaction_detail_report(
        query,
        None,
        Some(organization.id),
        None,
        None,
        0,
        100,
        connection,
    )
    .unwrap();
    let expected_results = vec![build_transaction_report_row(
        1,
        &user,
        &order,
        &event,
        &ticket_type,
        &fee_schedule_range,
        2,
        ticket_pricing.price_in_cents,
    )];
    assert_eq!(result.data, expected_results);
    assert_eq!(result.paging.total, 1);

    // With query, for organization (query finds order number)
    let query = order2.order_number();
    let result = Report::transaction_detail_report(
        Some(query),
        None,
        Some(organization.id),
        None,
        None,
        0,
        100,
        connection,
    )
    .unwrap();
    let expected_results = vec![build_transaction_report_row(
        1,
        &user2,
        &order2,
        &event,
        &ticket_type,
        &fee_schedule_range,
        2,
        ticket_pricing.price_in_cents,
    )];
    assert_eq!(result.data, expected_results);
    assert_eq!(result.paging.total, 1);

    // With query, for organization (query finds event name)
    let query = "Event2".to_string();
    let result = Report::transaction_detail_report(
        Some(query),
        None,
        Some(organization.id),
        None,
        None,
        0,
        100,
        connection,
    )
    .unwrap();
    let expected_results = vec![build_transaction_report_row(
        1,
        &user3,
        &order3,
        &event2,
        &ticket_type2,
        &fee_schedule_range2,
        2,
        ticket_pricing2.price_in_cents,
    )];
    assert_eq!(result.data, expected_results);
    assert_eq!(result.paging.total, 1);

    // With pagination
    let result = Report::transaction_detail_report(
        None,
        None,
        Some(organization.id),
        None,
        None,
        0,
        1,
        connection,
    )
    .unwrap();
    let expected_results = vec![build_transaction_report_row(
        4,
        &user,
        &order,
        &event,
        &ticket_type,
        &fee_schedule_range,
        2,
        ticket_pricing.price_in_cents,
    )];
    assert_eq!(result.data, expected_results);
    assert_eq!(result.paging.total, 4);

    // No query, for organization with time range
    let start = Utc::now().naive_utc() - Duration::days(4) - Duration::seconds(20);
    let end = Utc::now().naive_utc() - Duration::days(2) + Duration::seconds(20);
    let result = Report::transaction_detail_report(
        None,
        None,
        Some(organization.id),
        Some(start),
        Some(end),
        0,
        100,
        connection,
    )
    .unwrap();
    let expected_results = vec![
        build_transaction_report_row(
            3,
            &user2,
            &order2,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
        build_transaction_report_row(
            3,
            &user3,
            &order3,
            &event2,
            &ticket_type2,
            &fee_schedule_range2,
            2,
            ticket_pricing2.price_in_cents,
        ),
        build_transaction_report_row(
            3,
            &user4,
            &order4,
            &event,
            &ticket_type,
            &fee_schedule_range,
            2,
            ticket_pricing.price_in_cents,
        ),
    ];
    assert_eq!(result.data, expected_results);
    assert_eq!(result.paging.total, 3);
}

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

fn build_transaction_report_row(
    total: i64,
    user: &User,
    order: &Order,
    event: &Event,
    ticket_type: &TicketType,
    fee_schedule_range: &FeeScheduleRange,
    quantity: i64,
    price_per_ticket: i64,
) -> TransactionReportRow {
    TransactionReportRow {
        total,
        quantity,
        event_name: event.name.clone(),
        ticket_name: ticket_type.name.clone(),
        actual_quantity: quantity,
        refunded_quantity: 0,
        unit_price_in_cents: price_per_ticket,
        gross: (price_per_ticket
            + fee_schedule_range.client_fee_in_cents
            + fee_schedule_range.company_fee_in_cents)
            * quantity
            + event.company_fee_in_cents
            + event.client_fee_in_cents,
        company_fee_in_cents: fee_schedule_range.company_fee_in_cents,
        client_fee_in_cents: fee_schedule_range.client_fee_in_cents,
        gross_fee_in_cents: fee_schedule_range.company_fee_in_cents
            + fee_schedule_range.client_fee_in_cents,
        gross_fee_in_cents_total: (fee_schedule_range.company_fee_in_cents
            + fee_schedule_range.client_fee_in_cents)
            * quantity,
        event_fee_company_in_cents: event.company_fee_in_cents,
        event_fee_client_in_cents: event.client_fee_in_cents,
        event_fee_gross_in_cents: event.company_fee_in_cents + event.client_fee_in_cents,
        event_fee_gross_in_cents_total: event.company_fee_in_cents + event.client_fee_in_cents,
        fee_range_id: Some(fee_schedule_range.id),
        order_type: OrderTypes::Cart,
        payment_method: Some(PaymentMethods::External),
        payment_provider: Some("External".into()),
        transaction_date: order.paid_at.clone().unwrap(),
        redemption_code: None,
        order_id: order.id,
        event_id: event.id,
        user_id: user.id,
        first_name: user.first_name.clone().unwrap(),
        last_name: user.last_name.clone().unwrap(),
        email: user.email.clone().unwrap(),
        event_start: event.event_start,
    }
}
