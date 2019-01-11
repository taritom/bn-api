CREATE TABLE stages
(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    venue_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT NULL,
    capacity BIGINT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX index_stages_venue_id_name ON stages (
    venue_id,
    name
);

-- Indices
CREATE INDEX index_stages_venue_id ON events (venue_id);
