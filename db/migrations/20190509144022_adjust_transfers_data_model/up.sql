CREATE TABLE transfer_tickets (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  ticket_instance_id uuid NOT NULL REFERENCES ticket_instances(id) ON DELETE CASCADE,
  transfer_id uuid NOT NULL REFERENCES transfers(id) ON DELETE CASCADE,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX index_transfer_tickets_transfer_id ON transfer_tickets(transfer_id);
CREATE INDEX index_transfer_tickets_ticket_instance_id ON transfer_tickets(ticket_instance_id);

create temporary table transfer_migration_temp as
(
    SELECT transfer_key, min(id::text)::uuid as id
    FROM transfers
    GROUP BY transfer_key
);

INSERT INTO transfer_tickets(ticket_instance_id, transfer_id)
SELECT t.ticket_instance_id, tmt.id FROM transfer_migration_temp tmt
JOIN transfers t ON t.transfer_key = tmt.transfer_key;

DELETE FROM transfers where id = ANY (SELECT t.id FROM transfer_migration_temp tmt
JOIN transfers t ON t.transfer_key = tmt.transfer_key AND tmt.id <> t.id);

ALTER TABLE transfers
    DROP ticket_instance_id;

DROP INDEX IF EXISTS index_transfers_transfer_key;
CREATE UNIQUE INDEX index_transfers_transfer_key ON transfers(transfer_key);

ALTER TABLE transfers
    ADD transfer_message_type Text NULL;
ALTER TABLE transfers
    ADD transfer_address Text NULL;

CREATE TABLE order_transfers (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  order_id uuid NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
  transfer_id uuid NOT NULL REFERENCES transfers(id) ON DELETE CASCADE,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

INSERT INTO order_transfers(order_id, transfer_id)
SELECT DISTINCT oi.order_id, t.id
FROM transfers t
JOIN transfer_tickets tt ON tt.transfer_id = t.id
JOIN ticket_instances ti ON tt.ticket_instance_id = ti.id
JOIN order_items oi ON oi.id = ti.order_item_id
JOIN orders o ON o.id = oi.order_id
WHERE t.source_user_id = COALESCE(o.on_behalf_of_user_id, o.user_id);

CREATE UNIQUE INDEX index_order_transfers_order_id_transfer_id ON order_transfers(order_id, transfer_id);

DROP TABLE transfer_migration_temp;
