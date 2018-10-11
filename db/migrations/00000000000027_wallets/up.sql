CREATE TABLE wallets (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id uuid NULL REFERENCES users(id),
  organization_id uuid NULL REFERENCES organizations(id),
  name TEXT NOT NULL,
  secret_key TEXT NOT NULL,
  public_key TEXT NOT NULL,
  default_flag BOOLEAN NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now(),
  CHECK (user_id IS NOT NULL OR organization_id IS NOT NULL)
);

-- Indices
CREATE UNIQUE INDEX index_wallets_user_id_organization_id_name ON wallets(user_id, organization_id, name);
CREATE INDEX index_wallets_user_id ON wallets(user_id);
CREATE INDEX index_wallets_organization_id ON wallets(organization_id);

