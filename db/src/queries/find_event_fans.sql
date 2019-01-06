SELECT DISTINCT
    COALESCE(orders.on_behalf_of_user_id, orders.user_id) AS order_user_id,
    event_interest.user_id                                AS event_interest_user_id,
    events.organization_id                                AS organization_id,
    users.first_name                                      AS first_name,
    users.last_name                                       AS last_name,
    users.email                                           AS email,
    users.phone                                           AS phone,
    users.thumb_profile_pic_url                           AS thumb_profile_pic_url,
    users.id                                              AS user_id,
    count(distinct orders.id)                             AS order_count,
    users.created_at                                      AS created_at,
    min(orders.order_date)                                AS first_order_time,
    max(orders.order_date)                                AS last_order_time,
    cast(sum(order_items.unit_price_in_cents * order_items.quantity) as bigint) AS revenue_in_cents,
    count(*) over()                                       AS total_rows
FROM event_interest
       FULL OUTER JOIN orders ON COALESCE(orders.on_behalf_of_user_id, orders.user_id) = event_interest.user_id
       LEFT JOIN order_items ON orders.id = order_items.order_id
       LEFT JOIN users ON users.id = COALESCE(COALESCE(orders.on_behalf_of_user_id, orders.user_id), event_interest.user_id)
       LEFT JOIN events ON COALESCE(order_items.event_id, event_interest.event_id) = events.id
WHERE
    (event_interest.event_id = $1 OR order_items.event_id = $1) AND
    (
        users.first_name ILIKE $2 OR
        users.last_name  ILIKE $2 OR
        users.email      ILIKE $2 OR
        users.phone      ILIKE $2
    )
    -- (
    --     users.first_name ILIKE '%'||$2||'%' OR
    --     users.last_name ILIKE '%'||$2||'%' OR
    --     users.email ILIKE '%'||$2||'%' OR
    --     users.phone ILIKE '%'||$2||'%'
    -- )

GROUP BY
    (
        order_user_id,
        event_interest_user_id,
        users.first_name,
        users.last_name,
        users.email,
        users.phone,
        users.thumb_profile_pic_url,
        users.id,
        events.organization_id
    )
ORDER BY {sort_column} {sort_direction}
LIMIT $4
OFFSET $5;