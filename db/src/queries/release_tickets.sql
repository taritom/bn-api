UPDATE ticket_instances
SET
    order_item_id = NULL,
    reserved_until = NULL,
    updated_at = now()
WHERE id IN (SELECT t.id
             FROM ticket_instances AS t
                    INNER JOIN assets AS a ON t.asset_id = a.id
             WHERE t.order_item_id = $1
             LIMIT $2 FOR UPDATE SKIP LOCKED)
    RETURNING
      id,
      asset_id,
      token_id,
      ticket_holding_id,
      order_item_id,
      reserved_until,
      created_at,
      updated_at;
