SELECT
  u.id as operator_id,
  concat(u.first_name, ' ', u.last_name) as operator_name,
  e.name as event_name,
  e.event_start as event_date,
  o.external_payment_type,
  CAST(SUM(oi.quantity - oi.refunded_quantity) FILTER (WHERE oi.item_type = 'Tickets') AS BIGINT) as number_of_tickets,
  f.face_value_in_cents,
  CAST(COALESCE(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) FILTER (WHERE oi.item_type <> 'Tickets'), 0) AS BIGINT) as total_fees_in_cents,
  CAST(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) AS BIGINT) as total_sales_in_cents
FROM orders o
       LEFT JOIN order_items oi on o.id = oi.order_id
       LEFT JOIN events e on oi.event_id = e.id
       LEFT JOIN ticket_pricing tp on (oi.ticket_pricing_id = tp.id)
       JOIN users u on o.user_id = u.id
       JOIN (
         SELECT
           u.id as user_id,
           CAST(AVG(tp.price_in_cents) FILTER (WHERE tp.price_in_cents IS NOT NULL) AS BIGINT) as face_value_in_cents
         FROM orders o
                LEFT JOIN order_items oi on o.id = oi.order_id
                LEFT JOIN events e on oi.event_id = e.id
                LEFT JOIN ticket_pricing tp on (oi.ticket_pricing_id = tp.id)
                JOIN users u on o.user_id = u.id
         WHERE o.status = 'Paid'
           AND o.box_office_pricing = true
           AND e.organization_id = $1
           AND ($2 IS NULL OR o.paid_at >= $2)
           AND ($3 IS NULL OR o.paid_at <= $3)
         GROUP BY u.id
       ) as f on u.id = f.user_id
WHERE o.status = 'Paid'
  AND o.box_office_pricing = true
  AND e.organization_id = $1
  AND ($2 IS NULL OR o.paid_at >= $2)
  AND ($3 IS NULL OR o.paid_at <= $3)
GROUP BY operator_name, operator_id, event_name, event_date, o.external_payment_type, f.face_value_in_cents
ORDER BY operator_name, operator_id, event_date, event_name;
