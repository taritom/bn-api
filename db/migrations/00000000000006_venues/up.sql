-- Define the venues table
CREATE TABLE venues (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  region_id uuid NULL REFERENCES regions (id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  address TEXT,
  city TEXT,
  state TEXT,
  country TEXT,
  postal_code TEXT,
  phone TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_venues_name ON venues (name);
CREATE INDEX index_venues_region_id ON venues (region_id);
