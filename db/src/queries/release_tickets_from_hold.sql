WITH cte AS (SELECT t.id
             FROM ticket_instances AS t
                      INNER JOIN assets AS a ON t.asset_id = a.id
             WHERE t.hold_id = $1
               AND a.ticket_type_id = $2
               AND t.status IN ('Available', 'Reserved')
               -- Release available prior to reserved
             ORDER BY t.status, t.reserved_until
             LIMIT $3 FOR UPDATE SKIP LOCKED)
UPDATE ticket_instances
SET hold_id    = NULL,
    updated_at = now()
FROM cte
WHERE cte.id = ticket_instances.id RETURNING ticket_instances.*;

