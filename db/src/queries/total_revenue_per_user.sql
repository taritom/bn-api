SELECT CAST(SUM(revenue_in_cents) AS BIGINT) AS revenue_in_cents, user_id
FROM (
         SELECT COALESCE(CAST(SUM(oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity)) as BigInt), 0) AS revenue_in_cents,
                COALESCE(o.on_behalf_of_user_id, o.user_id)                                                     AS user_id
         FROM order_items oi
                  INNER JOIN orders o ON o.id = oi.order_id
                  INNER JOIN events e ON oi.event_id = e.id
         WHERE
               e.organization_id = $1
           AND ($2 IS NULL OR e.id = $2)
               AND o.status = $3
           AND COALESCE(o.on_behalf_of_user_id, o.user_id) = ANY ($4)
         GROUP BY o.on_behalf_of_user_id, o.user_id) user_values
GROUP BY user_values.user_id;