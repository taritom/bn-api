
SELECT t2.event_id,
       t2.Name,
       (SELECT min(tp.price_in_cents) FROM ticket_pricing tp WHERE tp.ticket_type_id = t2.id) AS min_price,
       (SELECT max(tp.price_in_cents) FROM ticket_pricing tp WHERE tp.ticket_type_id = t2.id) AS max_price,
       count(*)                                                                               AS total,
       sum(CASE
             WHEN ti.status = 'Purchased' AND ti.hold_id IS NULL THEN 1
             ELSE 0 END)                                                                      AS sold_unreserved,
       sum(CASE
             WHEN ti.status = 'Purchased' AND ti.hold_id IS NOT NULL THEN 1
             ELSE 0 END)                                                                      AS sold_held,
       sum(CASE WHEN ti.status = 'Available' AND ti.hold_id IS NULL THEN 1 ELSE 0 END)                               AS open,
       sum(CASE WHEN ti.hold_id IS NOT NULL THEN 1 ELSE 0 END)                                AS held,
       (SELECT cast(sum(oi.unit_price_in_cents * oi.quantity) as BIGINT)
        FROM order_items oi
               INNER JOIN orders o ON oi.order_id = o.id
        WHERE oi.ticket_type_id = t2.id
          AND o.status = 'Paid') as sales_total_in_cents
FROM ticket_instances ti
       INNER JOIN assets a
       INNER JOIN ticket_types t2
       INNER JOIN events e ON t2.event_id = e.id ON a.ticket_type_id = t2.id ON ti.asset_id = a.id
WHERE e.organization_id = $1
GROUP BY t2.name, t2.id, t2.event_id;

