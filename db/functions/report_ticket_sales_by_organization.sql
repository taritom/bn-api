DROP FUNCTION IF EXISTS report_ticket_sales_by_organization(UUID, TIMESTAMP, TIMESTAMP);
CREATE OR REPLACE FUNCTION report_ticket_sales_by_organization(p_organization_id UUID, p_start_date TIMESTAMP, p_end_date TIMESTAMP) RETURNS TABLE (
    id          UUID,
    first_name  TEXT,
    last_name   TEXT,
    email       TEXT,
    events      INT8,
    tickets     INT8,
    transferred INT8,
    gross_spent NUMERIC
)
AS
$body$
SELECT u.id,
       u.first_name,
       u.last_name,
       u.email,

       count(DISTINCT e.id)  AS events,
       count(DISTINCT ti.id) AS tickets,
       sum(CASE
               WHEN oi.item_type = 'Tickets' AND ti.wallet_id != w.id THEN 1
               ELSE 0 END)   AS transferred,
       (SELECT sum((oi2.quantity - oi2.refunded_quantity) * oi2.unit_price_in_cents)
        FROM orders o2
                 INNER JOIN order_items oi2
                            ON o2.id = oi2.order_id
        WHERE o2.user_id = u.id
          AND o2.status = 'Paid'
       )                     AS gross_spent
FROM orders o
         INNER JOIN order_items oi ON o.id = oi.order_id
         INNER JOIN events e ON e.id = oi.event_id
         INNER JOIN users u ON o.user_id = u.id
         LEFT JOIN ticket_instances ti ON (oi.id = ti.order_item_id AND ti.status IN ('Purchased', 'Redeemed'))
         LEFT JOIN refunded_tickets rt ON oi.id = rt.order_item_id AND rt.ticket_instance_id = ti.id
         INNER JOIN wallets w ON u.id = w.user_id
WHERE e.organization_id = p_organization_id
  AND o.status = 'Paid'
  AND o.paid_at >= p_start_date
  AND o.paid_at < p_end_date
  AND rt.id IS NULL
GROUP BY u.id, u.first_name, u.last_name, u.email
ORDER BY u.first_name, u.last_name;

$body$
    LANGUAGE SQL;

