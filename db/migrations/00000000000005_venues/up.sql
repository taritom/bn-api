-- Define the venues table
CREATE TABLE venues (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  name TEXT NOT NULL,
  address TEXT,
  city TEXT,
  state TEXT,
  country TEXT,
  zip TEXT,
  phone TEXT
);
