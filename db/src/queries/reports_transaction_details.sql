SELECT
       e.name AS event_name,
       CASE
         WHEN oi.ticket_type_id IS NULL THEN
           'Per Event Fees'
         ELSE
           tt.name END                    AS ticket_name,
       CAST(oi.quantity AS BIGINT),
       CAST(oi.unit_price_in_cents AS BIGINT),
       CAST(CASE
         WHEN oi.item_type = 'Tickets' THEN
           oi_fees.company_fee_in_cents
         ELSE oi.company_fee_in_cents END AS BIGINT) AS company_fee_in_cents,
       CAST(CASE
         WHEN oi.item_type = 'Tickets' THEN
           oi_fees.client_fee_in_cents
         ELSE oi.client_fee_in_cents END AS BIGINT) AS client_fee_in_cents,
       orders.paid_at                     AS transaction_date,
       orders.order_type,
       p.payment_method,
       h.redemption_code,
       orders.id                          AS order_id,
       oi.event_id,
       orders.user_id,
       CAST (0 AS BIGINT) AS gross
FROM orders
       LEFT JOIN order_items oi on orders.id = oi.order_id
       LEFT JOIN order_items oi_fees on oi.id = oi_fees.parent_id
       LEFT JOIN ticket_types tt ON (oi.ticket_type_id = tt.id)
       LEFT JOIN payments p on orders.id = p.order_id
       LEFT JOIN holds h on oi.hold_id = h.id
       LEFT JOIN events e on oi.event_id = e.id
WHERE orders.status = 'Paid'
AND ($1 IS NULL OR oi.event_id = $2)
AND ($3 IS NULL OR e.organization_id = $4)
  AND (oi.item_type = 'Tickets'
  OR oi.item_type = 'EventFees');