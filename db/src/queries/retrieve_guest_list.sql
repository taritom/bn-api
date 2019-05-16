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
                   OR NOW() >= e.redeem_date
                   OR NOW() >= e.event_start - INTERVAL '1 day 1 minute'
               THEN ti.redeem_key
           ELSE NULL END                                       AS redeem_key,
       ti.status                                               AS status,
       e.id                                                    AS event_id,
       e.name                                                  AS event_name,
       e.door_time                                             AS door_time,
       e.event_start                                           AS event_start,
       v.id                                                    AS venue_id,
       v.name                                                  AS venue_name,
       e.redeem_date                                           AS redeem_date,
       ti.updated_at                                           AS updated_at,
       CASE
           WHEN ti.redeemed_by_user_id IS NOT NULL THEN
               CONCAT(redeemer.first_name, ' ', redeemer.last_name)
           ELSE NULL END                                       AS redeemed_by,
       ti.redeemed_at                                          AS redeemed_at

FROM ticket_instances ti
         INNER JOIN assets a ON ti.asset_id = a.id
         INNER JOIN order_items oi ON ti.order_item_id = oi.id
         INNER JOIN orders o ON o.id = oi.order_id
         INNER JOIN ticket_types t2 ON a.ticket_type_id = t2.id
         INNER JOIN wallets w ON ti.wallet_id = w.id
         INNER JOIN users u ON  w.user_id = u.id
         INNER JOIN events e ON t2.event_id = e.id
         LEFT JOIN venues v ON e.venue_id = v.id
         LEFT JOIN users redeemer ON ti.redeemed_by_user_id = redeemer.id
WHERE ($1 IS NULL OR t2.event_id = $1)
  AND ($2 IS NULL OR u.first_name ILIKE '%' || $2 || '%'
    OR u.last_name ILIKE '%' || $2 || '%'
    OR u.email ILIKE '%' || $2 || '%'
    OR u.phone ILIKE '%' || $2 || '%')
  AND ($3 IS NULL OR ti.updated_at >= $3)
  AND ($4 IS NULL OR ti.id = $4)
ORDER BY u.last_name, ti.id
