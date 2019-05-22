ALTER TABLE transfers
    DROP transfer_address;
ALTER TABLE transfers
    DROP transfer_message_type;

DROP INDEX IF EXISTS index_order_transfers_order_id_transfer_id;
DROP TABLE IF EXISTS order_transfers;

DROP INDEX IF EXISTS index_transfers_transfer_key;
CREATE INDEX index_transfers_transfer_key ON transfers(transfer_key);

create temporary table transfer_migration_temp_revert as
(
    SELECT transfer_key, min(tt.ticket_instance_id::text)::uuid as ticket_instance_id
    FROM transfers t
    JOIN transfer_tickets tt ON tt.transfer_id = t.id
    GROUP BY transfer_key
);

ALTER TABLE transfers
    ADD ticket_instance_id uuid NULL REFERENCES ticket_instances(id);

UPDATE transfers
SET ticket_instance_id = tmtr.ticket_instance_id
FROM transfers t
INNER JOIN transfer_migration_temp_revert tmtr ON t.transfer_key = tmtr.transfer_key
WHERE transfers.transfer_key = t.transfer_key;

INSERT INTO transfers(ticket_instance_id, source_user_id, destination_user_id, transfer_expiry_date, transfer_key, status)
SELECT tt.ticket_instance_id, t.source_user_id, t.destination_user_id, t.transfer_expiry_date, t.transfer_key, t.status
FROM transfers t
JOIN transfer_tickets tt ON tt.transfer_id = t.id
JOIN transfer_migration_temp_revert tmtr ON t.transfer_key = tmtr.transfer_key
WHERE tmtr.ticket_instance_id <> tt.ticket_instance_id;

DROP INDEX IF EXISTS index_transfer_tickets_ticket_instance_id;
DROP INDEX IF EXISTS index_transfer_tickets_transfer_id;
DROP TABLE IF EXISTS transfer_tickets;

DROP TABLE transfer_migration_temp_revert;
