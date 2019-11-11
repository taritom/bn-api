SELECT oi.event_id,
       oi.ticket_type_id                                                                AS ticket_type_id,
       tp.name                                                                          AS pricing_name,
       tt.name                                                                          AS ticket_name,
       CAST(SUM(oi.quantity - oi.refunded_quantity) AS BIGINT)                                                 AS total_sold,
       CAST(
           COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE h.hold_type = 'Comp'), 0) AS BIGINT) AS comp_count,
       CAST(COALESCE(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE orders.box_office_pricing = false),
                     0) AS BIGINT)                                                      AS online_count,
       CAST(oi.unit_price_in_cents + COALESCE(oi_promo_code.unit_price_in_cents,
                           0) AS BIGINT)                                                AS price_in_cents, -- Face price, not actual price paid
       CAST(COALESCE(SUM(oi_fees.company_fee_in_cents * (oi_fees.quantity - oi_fees.refunded_quantity)), 0) AS BIGINT) AS total_company_fee_in_cents,
       CAST(COALESCE(oi_fees.company_fee_in_cents, 0) AS BIGINT) AS company_fee_in_cents,
       CAST(COALESCE(SUM(oi_fees.client_fee_in_cents * (oi_fees.quantity - oi_fees.refunded_quantity)), 0) AS BIGINT)  AS total_client_fee_in_cents,
       CAST(COALESCE(oi_fees.client_fee_in_cents, 0) AS BIGINT) AS client_fee_in_cents
FROM orders
       LEFT JOIN order_items oi on orders.id = oi.order_id
       LEFT JOIN order_items oi_fees ON (oi_fees.item_type = 'PerUnitFees' AND oi.id = oi_fees.parent_id)
       LEFT JOIN order_items oi_promo_code
                 ON (oi_promo_code.item_type = 'Discount' AND oi.id = oi_promo_code.parent_id)
       LEFT JOIN ticket_types tt ON (oi.ticket_type_id = tt.id)
       LEFT JOIN ticket_pricing tp ON (oi.ticket_pricing_id = tp.id)
       LEFT JOIN (SELECT order_id, ARRAY_TO_STRING(ARRAY_AGG(DISTINCT p.payment_method), ', ') AS payment_method FROM payments p GROUP BY p.payment_method, p.order_id) AS p on orders.id = p.order_id
       LEFT JOIN holds h on oi.hold_id = h.id
       LEFT JOIN events e on oi.event_id = e.id
WHERE orders.status = 'Paid'
  AND ($1 is null or oi.event_id = $1)
  AND ($2 is null or e.organization_id = $2)
  AND oi.item_type = 'Tickets'
  AND ($3 IS NULL OR orders.paid_at >= $3)
  AND ($4 IS NULL OR orders.paid_at <= $4)
GROUP BY oi.event_id, oi.ticket_type_id, tt.name, tp.name, oi.unit_price_in_cents, oi_promo_code.unit_price_in_cents, oi_fees.company_fee_in_cents, oi_fees.client_fee_in_cents;
