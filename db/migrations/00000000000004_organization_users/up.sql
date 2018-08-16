-- Define the organization_users table
CREATE TABLE organization_users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  organization_id uuid NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE
);

-- Indices
CREATE INDEX index_organization_users_organization_id_user_id ON organization_users (organization_id,user_id);
