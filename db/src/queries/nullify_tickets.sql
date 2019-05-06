WITH cte AS (SELECT t.id
             FROM ticket_instances AS t
             WHERE t.asset_id = $1
               AND t.status IN ('Available', 'Reserved')
               AND t.hold_id IS NULL
               -- Nullify Available inventory prior to Reserved
             ORDER BY t.status, t.reserved_until
             LIMIT $2 FOR UPDATE SKIP LOCKED)
UPDATE ticket_instances
SET status = 'Nullified'
FROM cte
WHERE ticket_instances.id = cte.id RETURNING ticket_instances.*;

