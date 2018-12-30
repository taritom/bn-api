SELECT CAST(AVG(oi.unit_price_in_cents) AS BIGINT)               AS unit_price_in_cents,
       CAST(COALESCE(SUM(oi.company_fee_in_cents), 0) AS BIGINT) AS total_company_fee_in_cents,
       CAST(COALESCE(AVG(oi.company_fee_in_cents), 0) AS BIGINT) AS company_fee_in_cents,
       CAST(COALESCE(SUM(oi.client_fee_in_cents), 0) AS BIGINT)  AS total_client_fee_in_cents,
       CAST(COALESCE(AVG(oi.client_fee_in_cents), 0) AS BIGINT)  AS client_fee_in_cents
FROM orders
       LEFT JOIN order_items oi on orders.id = oi.order_id
       LEFT JOIN events e on oi.event_id = e.id
WHERE orders.status = 'Paid'
  AND oi.event_id = $1
  AND e.organization_id = $2
  AND oi.item_type = 'EventFees'
  AND ($3 IS NULL OR orders.paid_at >= $3)
  AND ($4 IS NULL OR orders.paid_at <= $4);