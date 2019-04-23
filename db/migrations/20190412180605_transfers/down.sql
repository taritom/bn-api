ALTER TABLE ticket_instances
    ADD transfer_expiry_date TIMESTAMP NULL;
ALTER TABLE ticket_instances
    ADD transfer_key Uuid NULL;

CREATE INDEX index_ticket_instances_transfer_key ON ticket_instances(transfer_key);

DROP INDEX IF EXISTS index_transfers_ticket_instance_id;
DROP INDEX IF EXISTS index_transfers_source_user_id;
DROP INDEX IF EXISTS index_transfers_transfer_key;
DROP TABLE IF EXISTS transfers;
