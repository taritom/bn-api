CREATE TABLE ticket_instances (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  asset_id uuid NOT NULL REFERENCES assets(id),
  token_id INT NOT NULL,
  ticket_holding_id Uuid NULL REFERENCES ticket_holdings(id),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_ticket_instances_asset_id ON ticket_instances(asset_id);
CREATE INDEX index_ticket_instances_ticket_holding_id ON ticket_instances(ticket_holding_id);
CREATE INDEX index_ticket_instances_asset_id_token_id ON ticket_instances(asset_id, token_id);

