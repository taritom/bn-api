CREATE TABLE assets (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  blockchain_name TEXT NOT NULL,
  blockchain_asset_id TEXT NULL,
  status TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE UNIQUE INDEX index_assets_blockchain_asset_id ON assets(blockchain_asset_id);
CREATE UNIQUE INDEX index_assets_blockchain_name ON assets(blockchain_name);


