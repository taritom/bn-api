CREATE TABLE ticket_holdings (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  name TEXT NOT NULL,
  asset_id uuid NOT NULL REFERENCES assets(id),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_ticket_holdings_asset_id ON ticket_holdings(asset_id);
CREATE INDEX index_ticket_holdings_asset_id_name ON ticket_holdings(asset_id, name);

