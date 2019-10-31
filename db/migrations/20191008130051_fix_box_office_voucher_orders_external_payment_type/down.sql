UPDATE orders
SET external_payment_type = NULL
FROM (
  SELECT DISTINCT o.id
  FROM orders o
  JOIN order_items oi ON oi.order_id = o.id
  WHERE o.box_office_pricing = true
  AND o.status <> 'Draft'
  GROUP BY o.id
  HAVING sum(oi.unit_price_in_cents) = 0
) o
WHERE o.id = orders.id;
