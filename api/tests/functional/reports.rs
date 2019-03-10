use bigneon_db::dev::HoldBuilder;
use bigneon_db::prelude::*;
use chrono::prelude::*;
use functional::base;
use support::database::TestDatabase;

#[test]
pub fn ticket_counts_report() {
    let database = TestDatabase::new();
    let connection = database.connection.get();
    let user = database.create_user().finish();
    let organization = database.create_organization().finish();
    let venue = database.create_venue().finish();
    let event = database
        .create_event()
        .with_name("Event1".into())
        .with_event_start(NaiveDate::from_ymd(2016, 7, 8).and_hms(9, 10, 11))
        .with_event_end(NaiveDate::from_ymd(2016, 7, 9).and_hms(9, 10, 11))
        .with_venue(&venue)
        .with_organization(&organization)
        .with_ticket_pricing()
        .finish();

    let conn = database.connection.get();
    let mut cart = Order::find_or_create_cart(&user, conn).unwrap();
    let ticket_type = &event.ticket_types(true, None, conn).unwrap()[0];
    cart.update_quantities(
        user.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: None,
        }],
        false,
        false,
        conn,
    )
    .unwrap();

    let total = cart.calculate_total(conn).unwrap();
    cart.add_external_payment(
        Some("test".to_string()),
        ExternalPaymentType::Voucher,
        user.id,
        total,
        conn,
    )
    .unwrap();

    //----------Box Office
    let report =
        Report::ticket_sales_and_counts(None, None, None, None, true, true, true, true, connection)
            .unwrap();
    //Counts
    assert_eq!(report.counts[0].allocation_count_including_nullified, 100);
    assert_eq!(report.counts[0].allocation_count, 100);
    assert_eq!(report.counts[0].unallocated_count, 90);
    assert_eq!(report.counts[0].reserved_count, 0);
    assert_eq!(report.counts[0].redeemed_count, 0);
    assert_eq!(report.counts[0].purchased_count, 10);
    assert_eq!(report.counts[0].nullified_count, 0);
    assert_eq!(report.counts[0].available_for_purchase_count, 90);
    assert_eq!(report.counts[0].total_refunded_count, 0);
    assert_eq!(report.counts[0].comp_count, 0);
    assert_eq!(report.counts[0].comp_available_count, 0);
    assert_eq!(report.counts[0].comp_redeemed_count, 0);
    assert_eq!(report.counts[0].comp_purchased_count, 0);
    assert_eq!(report.counts[0].comp_reserved_count, 0);
    assert_eq!(report.counts[0].comp_nullified_count, 0);
    assert_eq!(report.counts[0].hold_count, 0);
    assert_eq!(report.counts[0].hold_available_count, 0);
    assert_eq!(report.counts[0].hold_redeemed_count, 0);
    assert_eq!(report.counts[0].hold_purchased_count, 0);
    assert_eq!(report.counts[0].hold_reserved_count, 0);
    assert_eq!(report.counts[0].hold_nullified_count, 0);
    //Sales
    assert_eq!(report.sales[0].box_office_order_count, 1);
    assert_eq!(report.sales[0].online_order_count, 0);
    assert_eq!(report.sales[0].box_office_refunded_count, 0);
    assert_eq!(report.sales[0].online_refunded_count, 0);
    assert_eq!(report.sales[0].box_office_sales_in_cents, 1500);
    assert_eq!(report.sales[0].online_sales_in_cents, 0);
    assert_eq!(report.sales[0].box_office_sale_count, 10);
    assert_eq!(report.sales[0].online_sale_count, 0);
    assert_eq!(report.sales[0].comp_sale_count, 0);
    assert_eq!(report.sales[0].total_box_office_fees_in_cents, 500);
    assert_eq!(report.sales[0].total_online_fees_in_cents, 0);
    assert_eq!(report.sales[0].company_box_office_fees_in_cents, 200);
    assert_eq!(report.sales[0].client_box_office_fees_in_cents, 300);
    assert_eq!(report.sales[0].company_online_fees_in_cents, 0);
    assert_eq!(report.sales[0].client_online_fees_in_cents, 0);

    let mut cart = Order::find_or_create_cart(&user, conn).unwrap();
    let ticket_type = &event.ticket_types(true, None, conn).unwrap()[0];
    cart.update_quantities(
        user.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 10,
            redemption_code: None,
        }],
        false,
        false,
        conn,
    )
    .unwrap();

    //----------Online
    cart.add_provider_payment(
        Some("Using Payment Provider".to_string()),
        PaymentProviders::Stripe,
        Some(user.id),
        total,
        PaymentStatus::Completed,
        None,
        json!({}),
        connection,
    )
    .unwrap();

    let report =
        Report::ticket_sales_and_counts(None, None, None, None, true, true, true, true, connection)
            .unwrap();
    //Counts
    assert_eq!(report.counts[0].allocation_count_including_nullified, 100);
    assert_eq!(report.counts[0].allocation_count, 100);
    assert_eq!(report.counts[0].unallocated_count, 80);
    assert_eq!(report.counts[0].reserved_count, 0);
    assert_eq!(report.counts[0].redeemed_count, 0);
    assert_eq!(report.counts[0].purchased_count, 20);
    assert_eq!(report.counts[0].nullified_count, 0);
    assert_eq!(report.counts[0].available_for_purchase_count, 80);
    assert_eq!(report.counts[0].total_refunded_count, 0);
    assert_eq!(report.counts[0].comp_count, 0);
    assert_eq!(report.counts[0].comp_available_count, 0);
    assert_eq!(report.counts[0].comp_redeemed_count, 0);
    assert_eq!(report.counts[0].comp_purchased_count, 0);
    assert_eq!(report.counts[0].comp_reserved_count, 0);
    assert_eq!(report.counts[0].comp_nullified_count, 0);
    assert_eq!(report.counts[0].hold_count, 0);
    assert_eq!(report.counts[0].hold_available_count, 0);
    assert_eq!(report.counts[0].hold_redeemed_count, 0);
    assert_eq!(report.counts[0].hold_purchased_count, 0);
    assert_eq!(report.counts[0].hold_reserved_count, 0);
    assert_eq!(report.counts[0].hold_nullified_count, 0);

    //Sales
    assert_eq!(report.sales[0].box_office_order_count, 1);
    assert_eq!(report.sales[0].online_order_count, 1);
    assert_eq!(report.sales[0].box_office_refunded_count, 0);
    assert_eq!(report.sales[0].online_refunded_count, 0);
    assert_eq!(report.sales[0].box_office_sales_in_cents, 1500);
    assert_eq!(report.sales[0].online_sales_in_cents, 1500);
    assert_eq!(report.sales[0].box_office_sale_count, 10);
    assert_eq!(report.sales[0].online_sale_count, 10);
    assert_eq!(report.sales[0].comp_sale_count, 0);
    assert_eq!(report.sales[0].total_box_office_fees_in_cents, 500);
    assert_eq!(report.sales[0].total_online_fees_in_cents, 500);
    assert_eq!(report.sales[0].company_box_office_fees_in_cents, 200);
    assert_eq!(report.sales[0].client_box_office_fees_in_cents, 300);
    assert_eq!(report.sales[0].company_online_fees_in_cents, 200);
    assert_eq!(report.sales[0].client_online_fees_in_cents, 300);

    //----------Comps
    let redemption_code = HoldBuilder::new(connection)
        .with_name("Comp Hold".to_string())
        .with_ticket_type_id(ticket_type.id.clone())
        .with_hold_type(HoldTypes::Comp)
        .with_quantity(10)
        .finish()
        .redemption_code;

    let mut cart = Order::find_or_create_cart(&user, conn).unwrap();
    let ticket_type = &event.ticket_types(true, None, conn).unwrap()[0];
    cart.update_quantities(
        user.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 5,
            redemption_code: redemption_code,
        }],
        false,
        false,
        conn,
    )
    .unwrap();

    cart.add_provider_payment(
        Some("Using Payment Provider".to_string()),
        PaymentProviders::Stripe,
        Some(user.id),
        total,
        PaymentStatus::Completed,
        None,
        json!({}),
        connection,
    )
    .unwrap();

    let report = Report::ticket_sales_and_counts(
        None, None, None, None, true, true, false, true, connection,
    )
    .unwrap();

    //Counts
    assert_eq!(report.counts[0].allocation_count_including_nullified, 100);
    assert_eq!(report.counts[0].allocation_count, 100);
    assert_eq!(report.counts[0].unallocated_count, 75);
    assert_eq!(report.counts[0].reserved_count, 0);
    assert_eq!(report.counts[0].redeemed_count, 0);
    assert_eq!(report.counts[0].purchased_count, 25);
    assert_eq!(report.counts[0].nullified_count, 0);
    assert_eq!(report.counts[0].available_for_purchase_count, 70);
    assert_eq!(report.counts[0].total_refunded_count, 0);
    assert_eq!(report.counts[0].comp_count, 10);
    assert_eq!(report.counts[0].comp_available_count, 5);
    assert_eq!(report.counts[0].comp_redeemed_count, 0);
    assert_eq!(report.counts[0].comp_purchased_count, 5);
    assert_eq!(report.counts[0].comp_reserved_count, 0);
    assert_eq!(report.counts[0].comp_nullified_count, 0);
    assert_eq!(report.counts[0].hold_count, 0);
    assert_eq!(report.counts[0].hold_available_count, 0);
    assert_eq!(report.counts[0].hold_redeemed_count, 0);
    assert_eq!(report.counts[0].hold_purchased_count, 0);
    assert_eq!(report.counts[0].hold_reserved_count, 0);
    assert_eq!(report.counts[0].hold_nullified_count, 0);

    //Sales
    assert_eq!(report.sales[0].box_office_order_count, 1);
    assert_eq!(report.sales[0].online_order_count, 2);
    assert_eq!(report.sales[0].box_office_refunded_count, 0);
    assert_eq!(report.sales[0].online_refunded_count, 0);
    assert_eq!(report.sales[0].box_office_sales_in_cents, 1500);
    assert_eq!(report.sales[0].online_sales_in_cents, 1500);
    assert_eq!(report.sales[0].box_office_sale_count, 10);
    assert_eq!(report.sales[0].online_sale_count, 10);
    assert_eq!(report.sales[0].comp_sale_count, 5);
    assert_eq!(report.sales[0].total_box_office_fees_in_cents, 500);
    assert_eq!(report.sales[0].total_online_fees_in_cents, 500);
    assert_eq!(report.sales[0].company_box_office_fees_in_cents, 200);
    assert_eq!(report.sales[0].client_box_office_fees_in_cents, 300);
    assert_eq!(report.sales[0].company_online_fees_in_cents, 200);
    assert_eq!(report.sales[0].client_online_fees_in_cents, 300);

    //----------Holds
    let redemption_code = HoldBuilder::new(connection)
        .with_name("Normal Hold".to_string())
        .with_ticket_type_id(ticket_type.id.clone())
        .with_hold_type(HoldTypes::Discount)
        .with_quantity(10)
        .finish()
        .redemption_code;

    let mut cart = Order::find_or_create_cart(&user, conn).unwrap();
    let ticket_type = &event.ticket_types(true, None, conn).unwrap()[0];
    cart.update_quantities(
        user.id,
        &[UpdateOrderItem {
            ticket_type_id: ticket_type.id,
            quantity: 5,
            redemption_code: redemption_code,
        }],
        false,
        false,
        conn,
    )
    .unwrap();

    cart.add_provider_payment(
        Some("Using Payment Provider".to_string()),
        PaymentProviders::Stripe,
        Some(user.id),
        total,
        PaymentStatus::Completed,
        None,
        json!({}),
        connection,
    )
    .unwrap();

    let report = Report::ticket_sales_and_counts(
        None, None, None, None, true, true, false, true, connection,
    )
    .unwrap();

    //Counts
    assert_eq!(report.counts[0].allocation_count_including_nullified, 100);
    assert_eq!(report.counts[0].allocation_count, 100);
    assert_eq!(report.counts[0].unallocated_count, 70);
    assert_eq!(report.counts[0].reserved_count, 0);
    assert_eq!(report.counts[0].redeemed_count, 0);
    assert_eq!(report.counts[0].purchased_count, 30);
    assert_eq!(report.counts[0].nullified_count, 0);
    assert_eq!(report.counts[0].available_for_purchase_count, 60);
    assert_eq!(report.counts[0].total_refunded_count, 0);
    assert_eq!(report.counts[0].comp_count, 10);
    assert_eq!(report.counts[0].comp_available_count, 5);
    assert_eq!(report.counts[0].comp_redeemed_count, 0);
    assert_eq!(report.counts[0].comp_purchased_count, 5);
    assert_eq!(report.counts[0].comp_reserved_count, 0);
    assert_eq!(report.counts[0].comp_nullified_count, 0);
    assert_eq!(report.counts[0].hold_count, 10);
    assert_eq!(report.counts[0].hold_available_count, 5);
    assert_eq!(report.counts[0].hold_redeemed_count, 0);
    assert_eq!(report.counts[0].hold_purchased_count, 5);
    assert_eq!(report.counts[0].hold_reserved_count, 0);
    assert_eq!(report.counts[0].hold_nullified_count, 0);

    //Sales
    assert_eq!(report.sales[0].box_office_order_count, 1);
    assert_eq!(report.sales[0].online_order_count, 3);
    assert_eq!(report.sales[0].box_office_refunded_count, 0);
    assert_eq!(report.sales[0].online_refunded_count, 0);
    assert_eq!(report.sales[0].box_office_sales_in_cents, 1500);
    assert_eq!(report.sales[0].online_sales_in_cents, 2200);
    assert_eq!(report.sales[0].box_office_sale_count, 10);
    assert_eq!(report.sales[0].online_sale_count, 15);
    assert_eq!(report.sales[0].comp_sale_count, 5);
    assert_eq!(report.sales[0].total_box_office_fees_in_cents, 500);
    assert_eq!(report.sales[0].total_online_fees_in_cents, 750);
    assert_eq!(report.sales[0].company_box_office_fees_in_cents, 200);
    assert_eq!(report.sales[0].client_box_office_fees_in_cents, 300);
    assert_eq!(report.sales[0].company_online_fees_in_cents, 300);
    assert_eq!(report.sales[0].client_online_fees_in_cents, 450);

    //------Test a refund
    // Refund first ticket and event fee (leaving one ticket + one fee item for that ticket)
    //    let refund_items = vec![
    //        RefundItem {
    //            order_item_id: order_item.id,
    //            ticket_instance_id: Some(ticket.id),
    //        },
    //
    //    ];
    //    let json = Json(RefundAttributes {
    //        items: refund_items,
    //    });
    //
    //    let test_request = TestRequest::create();
    //    let mut path = Path::<PathParameters>::extract(&test_request.request).unwrap();
    //    path.id = cart.id;
    //    let response: HttpResponse = orders::refund((
    //        database.connection.clone(),
    //        path,
    //        json,
    //        auth_user,
    //        test_request.extract_state(),
    //    ))
    //        .into();
}

#[cfg(test)]
mod box_office_sales_summary_tests {
    use super::*;
    #[test]
    fn box_office_sales_summary_org_member() {
        base::reports::box_office_sales_summary(Roles::OrgMember, false);
    }
    #[test]
    fn box_office_sales_summary_admin() {
        base::reports::box_office_sales_summary(Roles::Admin, true);
    }
    #[test]
    fn box_office_sales_summary_user() {
        base::reports::box_office_sales_summary(Roles::User, false);
    }
    #[test]
    fn box_office_sales_summary_org_owner() {
        base::reports::box_office_sales_summary(Roles::OrgOwner, true);
    }
    #[test]
    fn box_office_sales_summary_door_person() {
        base::reports::box_office_sales_summary(Roles::DoorPerson, false);
    }
    #[test]
    fn box_office_sales_summary_promoter() {
        base::reports::box_office_sales_summary(Roles::Promoter, false);
    }
    #[test]
    fn box_office_sales_summary_promoter_read_only() {
        base::reports::box_office_sales_summary(Roles::PromoterReadOnly, false);
    }
    #[test]
    fn box_office_sales_summary_org_admin() {
        base::reports::box_office_sales_summary(Roles::OrgAdmin, true);
    }
    #[test]
    fn box_office_sales_summary_box_office() {
        base::reports::box_office_sales_summary(Roles::OrgBoxOffice, false);
    }
}
