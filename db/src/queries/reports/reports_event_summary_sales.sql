SELECT event_id,
       hold_id,
       hold_name,
       promo_redemption_code,
       ticket_type_id,
       total_sold,
       comp_count,
       box_office_count,
       online_count,
       price_in_cents,
       total_company_fee_in_cents,
       total_client_fee_in_cents,
       pricing_name,
       ticket_name,
       CAST(total_net_income + total_company_fee_in_cents +
            total_client_fee_in_cents AS BIGINT) AS total_gross_income_in_cents
FROM (
         SELECT oi.event_id,
                COALESCE(h.id, c.id)                   AS hold_id,
                COALESCE(h.name, c.name)               AS hold_name,
                c.redemption_code                      AS promo_redemption_code,
                oi.ticket_type_id,
                CAST(COALESCE(
                    SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE h.hold_type is null or h.hold_type != 'Comp'),
                    0) AS BIGINT)                      AS total_sold,
                CAST(
                    COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE h.hold_type = 'Comp'),
                             0) AS BIGINT)             AS comp_count,
                CAST(COALESCE(SUM(oi.quantity - oi.refunded_quantity)
                                  FILTER (WHERE orders.on_behalf_of_user_id IS NOT NULL and
                                                (h.hold_type is null or h.hold_type != 'Comp')),
                              0) AS BIGINT)            AS box_office_count,
                CAST(COALESCE(SUM(oi.quantity - oi.refunded_quantity)
                                  FILTER (WHERE orders.on_behalf_of_user_id IS NULL and
                                                (h.hold_type is null or h.hold_type != 'Comp')),
                              0) AS BIGINT)            AS online_count,
                CAST(oi.unit_price_in_cents + COALESCE(oi_promo_code.unit_price_in_cents,
                              0) AS BIGINT) AS price_in_cents, -- face price
                CAST(COALESCE(SUM((oi_fees.quantity - oi_fees.refunded_quantity) * oi_fees.company_fee_in_cents),
                              0) AS BIGINT)            AS total_company_fee_in_cents,
                CAST(COALESCE(SUM((oi_fees.quantity - oi_fees.refunded_quantity) * oi_fees.client_fee_in_cents),
                              0) AS BIGINT)            AS total_client_fee_in_cents,
                CAST(COALESCE(SUM(((oi.quantity - oi.refunded_quantity) * oi.unit_price_in_cents)) + SUM(
                    ((COALESCE(oi_promo_code.quantity, 0) - COALESCE(oi_promo_code.refunded_quantity, 0)) *
                     COALESCE(oi_promo_code.unit_price_in_cents, 0))),
                              0) AS BIGINT)            AS total_net_income,
                tp.name                                AS pricing_name,
                tt.name                                AS ticket_name
         FROM orders
                  LEFT JOIN order_items oi ON orders.id = oi.order_id
                  LEFT JOIN order_items oi_fees ON (oi_fees.item_type = 'PerUnitFees' AND oi.id = oi_fees.parent_id)
                  LEFT JOIN order_items oi_promo_code
                            ON (oi_promo_code.item_type = 'Discount' AND oi.id = oi_promo_code.parent_id)
                  LEFT JOIN codes c ON oi.code_id = c.id
                  LEFT JOIN ticket_types tt ON (oi.ticket_type_id = tt.id)
                  LEFT JOIN ticket_pricing tp ON (oi.ticket_pricing_id = tp.id)
                  LEFT JOIN holds h ON oi.hold_id = h.id
                  LEFT JOIN events e ON oi.event_id = e.id
         WHERE orders.status = 'Paid'
           AND ($1 IS NULL OR oi.event_id = $1)
           AND ($2 IS NULL OR e.organization_id = $2)
           AND oi.item_type = 'Tickets'
           AND ($3 IS NULL OR orders.paid_at >= $3)
           AND ($4 IS NULL OR orders.paid_at <= $4)
         GROUP BY oi.event_id, oi.ticket_type_id, tt.name, tp.name, oi.unit_price_in_cents,
                  oi_promo_code.unit_price_in_cents, h.id, h.name, c.id
     ) AS report_data;
