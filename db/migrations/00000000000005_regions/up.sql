CREATE TABLE regions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  name TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE UNIQUE INDEX index_regions_name ON regions (name);
