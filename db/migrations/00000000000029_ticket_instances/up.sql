CREATE TABLE ticket_instances (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  asset_id uuid NOT NULL REFERENCES assets(id),
  token_id INT NOT NULL,
  ticket_holding_id Uuid NULL REFERENCES ticket_holdings(id),
  order_item_id Uuid NULL REFERENCES order_items(id),
  wallet_id Uuid NULL REFERENCES wallets(id),
  reserved_until TIMESTAMP NULL,
  redeem_key Text NULL,
  status TEXT NOT NULL DEFAULT 'Available',
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_ticket_instances_asset_id ON ticket_instances(asset_id);
CREATE INDEX index_ticket_instances_order_item_id ON ticket_instances(order_item_id);
CREATE INDEX index_ticket_instances_ticket_holding_id ON ticket_instances(ticket_holding_id);
CREATE INDEX index_ticket_instances_asset_id_token_id ON ticket_instances(asset_id, token_id);
CREATE INDEX index_ticket_instances_redeem_key  ON ticket_instances(redeem_key);
CREATE INDEX index_ticket_instances_wallet_id ON ticket_instances(wallet_id);

