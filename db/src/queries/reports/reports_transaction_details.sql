SELECT e.name                                               AS event_name,
       COALESCE(tt.name, 'Per Event Fees')                  AS ticket_name,
       CAST(oi.quantity AS BIGINT)                          AS quantity,
       CAST(COALESCE(oi.refunded_quantity, 0) AS BIGINT)    AS refunded_quantity,
       CAST(oi.unit_price_in_cents AS BIGINT)               AS unit_price_in_cents,
       CAST(COALESCE(oi.company_fee_in_cents, 0) AS BIGINT) AS company_fee_in_cents,
       CAST(COALESCE(oi.client_fee_in_cents, 0) AS BIGINT)  AS client_fee_in_cents,
       orders.paid_at                                       AS transaction_date,
       orders.order_type,
       p.payment_method,
       h.redemption_code,
       orders.id                                            AS order_id,
       oi.event_id,
       orders.user_id,
       CAST (
         (oi.quantity - oi.refunded_quantity) * oi.unit_price_in_cents +
         (COALESCE(oi_fees.quantity, 0) - COALESCE(oi_fees.refunded_quantity, 0)) * COALESCE(oi_fees.unit_price_in_cents, 0)
       AS BIGINT) AS gross
FROM orders
       LEFT JOIN order_items oi on orders.id = oi.order_id
       LEFT JOIN order_items oi_fees on oi.id = oi_fees.parent_id
       LEFT JOIN ticket_types tt ON (oi.ticket_type_id = tt.id)
       LEFT JOIN payments p on orders.id = p.order_id
       LEFT JOIN holds h on oi.hold_id = h.id
       LEFT JOIN events e on oi.event_id = e.id
WHERE orders.status = 'Paid'
  AND ($1 IS NULL OR oi.event_id = $1)
  AND ($2 IS NULL OR e.organization_id = $2)
  AND ($3 IS NULL OR orders.paid_at >= $3)
  AND ($4 IS NULL OR orders.paid_at <= $4)
  AND (oi.item_type = 'Tickets' OR oi.item_type = 'EventFees')
