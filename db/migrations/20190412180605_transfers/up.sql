CREATE TABLE transfers (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  ticket_instance_id uuid NOT NULL REFERENCES ticket_instances(id),
  source_user_id uuid NOT NULL REFERENCES users(id),
  destination_user_id uuid NULL REFERENCES users(id),
  transfer_expiry_date TIMESTAMP NOT NULL,
  transfer_key Uuid NOT NULL,
  status VARCHAR(20) NOT NULL DEFAULT 'Pending',
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

insert into transfers (ticket_instance_id, source_user_id,  transfer_expiry_date, transfer_key, status)
select ti.id, w.user_id, ti.transfer_expiry_date, ti.transfer_key, 'Pending'
from ticket_instances ti
join wallets w on w.id = ti.wallet_id
where transfer_key is not null;

DROP INDEX IF EXISTS index_ticket_instances_transfer_key;
ALTER TABLE ticket_instances
    DROP COLUMN transfer_key;
ALTER TABLE ticket_instances
    DROP COLUMN transfer_expiry_date;

CREATE INDEX index_transfers_transfer_key ON transfers(transfer_key);
CREATE INDEX index_transfers_ticket_instance_id ON transfers(ticket_instance_id);
CREATE INDEX index_transfers_source_user_id ON transfers(source_user_id);
