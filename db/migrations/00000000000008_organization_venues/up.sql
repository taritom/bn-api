-- Define the organization_venues table
CREATE TABLE organization_venues (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  organization_id uuid NOT NULL REFERENCES organizations (id),
  venue_id uuid NOT NULL REFERENCES venues (id)
);

-- Indices
CREATE UNIQUE INDEX index_organization_venues_organization_id_venue_id ON organization_venues (organization_id,venue_id);
