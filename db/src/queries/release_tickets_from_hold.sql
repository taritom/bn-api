UPDATE ticket_instances
SET
    hold_id   = null,
    updated_at = now()
WHERE id IN (SELECT t.id
             FROM ticket_instances AS t
                    INNER JOIN assets AS a ON t.asset_id = a.id
             WHERE t.hold_id = $1
             AND a.ticket_type_id = $2
             AND (t.status = 'Available' or t.reserved_until < now())
             AND t.status <> 'Purchased'
             LIMIT $3 FOR UPDATE SKIP LOCKED)
    RETURNING
      id,
      asset_id,
      token_id,
      hold_id,
      order_item_id,
      wallet_id,
      reserved_until,
      status,
      redeem_key,
      transfer_key,
      transfer_expiry_date,
      created_at,
      updated_at,
      code_id;
