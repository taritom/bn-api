SELECT e.id,
       e.name,
       e.organization_id,
       v.id                      AS venue_id,
       v.name                    AS venue_name,
       v.address                 AS venue_address,
       v.city                    AS venue_city,
       v.state                   AS venue_state,
       v.country                 AS venue_country,
       v.postal_code             AS venue_postal_code,
       v.phone                   AS venue_phone,
       v.timezone                AS venue_timezone,
       e.created_at,
       e.event_start,
       e.door_time,
       e.event_end,
       e.status,
       e.publish_date,
       e.promo_image_url,
       e.additional_info,
       e.top_line_info,
       e.age_limit,
       e.cancelled_at,
       e.is_external,
       e.external_url,
       e.override_status,
       e.event_type,
       e.slug,
       (SELECT min(tp.start_date)
        FROM ticket_pricing tp
                 INNER JOIN ticket_types t2 ON tp.ticket_type_id = t2.id
        WHERE t2.event_id
                  = e.id)
                                 AS on_sale,
       (SELECT min(tp.price_in_cents)
        FROM ticket_pricing tp
                 INNER JOIN ticket_types t2 ON tp.ticket_type_id = t2.id
        WHERE t2.event_id
                  = e.id)
                                 AS min_price,
       (SELECT max(tp.price_in_cents)
        FROM ticket_pricing tp
                 INNER JOIN ticket_types t2 ON tp.ticket_type_id = t2.id
        WHERE t2.event_id
                  = e.id)
                                 AS max_price,
       (SELECT CAST(
                   SUM((oi.unit_price_in_cents * (oi.quantity - oi.refunded_quantity))
                       + (COALESCE(oi_promo_code.unit_price_in_cents, 0) * (COALESCE(oi_promo_code.quantity, 0) - COALESCE(oi_promo_code.refunded_quantity, 0)))) AS BIGINT)
        FROM order_items oi
                 LEFT JOIN order_items oi_promo_code ON oi_promo_code.item_type = 'Discount' AND oi_promo_code.parent_id = oi.id
                 INNER JOIN orders o ON oi.order_id = o.id
        WHERE oi.event_id = e.id
          AND oi.item_type = 'Tickets'
          AND o.status = 'Paid') AS sales_total_in_cents,
      (SELECT NOT EXISTS(SELECT 1 from order_items oi WHERE oi.event_id = e.id LIMIT 1)) AS eligible_for_deletion
FROM events e
       LEFT JOIN venues v ON e.venue_id = v.id
WHERE e.organization_id = $1
  AND e.deleted_at is null
  AND CASE
        WHEN $2 IS NULL THEN TRUE -- All events
        WHEN $2 THEN e.event_start >= now() OR e.event_end > now() -- upcoming
        ELSE e.event_end <= now() END -- past
  AND ($5 IS NULL OR e.id = ANY($5))
ORDER BY CASE WHEN $2 THEN e.event_start END ASC, CASE WHEN NOT $2 THEN e.event_start END DESC
LIMIT $4
OFFSET $3;
