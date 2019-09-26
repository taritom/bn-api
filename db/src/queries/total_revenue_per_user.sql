SELECT CAST(SUM(revenue_in_cents) AS BIGINT) AS revenue_in_cents,
       user_id,
       MAX(last_order_time)                  AS last_order_time,
       MIN(first_order_time)                 AS first_order_time,
       CAST(MAX(order_count) AS BIGINT)      AS order_count
FROM (
         SELECT COALESCE(CAST(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) as BigInt),
                         0)                                 AS revenue_in_cents,
                COALESCE(o.on_behalf_of_user_id, o.user_id) AS user_id,
                MAX(o.order_date)                           AS last_order_time,
                MIN(o.order_date)                           AS first_order_time,
                COUNT(DISTINCT o.id)                        AS order_count
         FROM order_items oi
                  INNER JOIN orders o ON o.id = oi.order_id
                  INNER JOIN events e ON oi.event_id = e.id
         WHERE e.organization_id = $1
           AND o.status = $2
           AND COALESCE(o.on_behalf_of_user_id, o.user_id) = ANY ($3)
         GROUP BY COALESCE(o.on_behalf_of_user_id, o.user_id)
     ) user_values
GROUP BY user_values.user_id;