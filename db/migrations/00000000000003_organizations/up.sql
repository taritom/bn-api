-- Define the organizations table
CREATE TABLE organizations (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  owner_user_id uuid NOT NULL REFERENCES users (id),
  name TEXT NOT NULL,
  address TEXT,
  city TEXT,
  state TEXT,
  country TEXT,
  zip TEXT,
  phone TEXT
);

-- Indices
CREATE INDEX index_organizations_owner_user_id ON organizations (owner_user_id);
