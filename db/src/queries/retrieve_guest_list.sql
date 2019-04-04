SELECT ti.id,
       t2.name                                                 AS ticket_type,
       u.first_name                                            AS first_name,
       u.last_name                                             AS last_name,
       u.id                                                    AS user_id,
       oi.order_id                                             AS order_id,
       oi.id                                                   AS order_item_id,
       cast(oi.unit_price_in_cents + coalesce((
                                                  SELECT SUM(unit_price_in_cents)
                                                  FROM order_items
                                                  WHERE parent_id = ti.order_item_id
                                              ), 0) AS BIGINT) AS price_in_cents,
       u.email                                                 AS email,
       u.phone                                                 AS phone,
       CASE
           WHEN
                   e.redeem_date IS NULL
                   OR NOW() > e.redeem_date
                   OR NOW() > e.event_start - INTERVAL '1 day'
               THEN ti.redeem_key
           ELSE NULL END                                         AS redeem_key,
       ti.status,
       e.id                                                    AS event_id,
       e.name                                                  AS event_name,
       e.door_time                                             AS door_time,
       e.event_start                                           AS event_start,
       v.id                                                    AS venue_id,
       v.name                                                  AS venue_name,
       e.redeem_date                                           AS redeem_date

FROM ticket_instances ti
         INNER JOIN assets a ON ti.asset_id = a.id
         INNER JOIN order_items oi ON ti.order_item_id = oi.id
         INNER JOIN orders o ON o.id = oi.order_id
         INNER JOIN ticket_types t2 ON a.ticket_type_id = t2.id
         INNER JOIN wallets w ON ti.wallet_id = w.id
         INNER JOIN users u ON coalesce(o.on_behalf_of_user_id, w.user_id) = u.id
         INNER JOIN events e ON t2.event_id = e.id
         INNER JOIN venues v ON e.venue_id = v.id
WHERE t2.event_id = $1
  AND (u.first_name ILIKE '%' || $2 || '%'
    OR u.last_name ILIKE '%' || $2 || '%'
    OR u.email ILIKE '%' || $2 || '%'
    OR u.phone ILIKE '%' || $2 || '%')
ORDER BY u.last_name, ti.id