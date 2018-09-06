ALTER TABLE ticket_types
  ADD asset_id Uuid NOT NULL REFERENCES assets;

CREATE INDEX index_ticket_types_asset_id ON ticket_types (asset_id);