WITH cte AS (SELECT t.id
             FROM ticket_instances AS t
                      INNER JOIN assets AS a ON t.asset_id = a.id
             WHERE t.order_item_id = $1
               AND t.status = ANY ($3)
               AND t.id = COALESCE($4, t.id)
             LIMIT $2 FOR UPDATE OF t SKIP LOCKED)
UPDATE ticket_instances
SET order_item_id  = NULL,
    reserved_until = NULL,
    redeem_key     = NULL,
    status         = $5,
    updated_at     = now()
FROM cte
WHERE cte.id = ticket_instances.id RETURNING ticket_instances.*;

