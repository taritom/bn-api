CREATE TABLE regions (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  name TEXT NOT NULL
);

-- Indices
CREATE UNIQUE INDEX index_regions_name ON regions (name);
