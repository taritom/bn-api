
CREATE TABLE ticket_allocations (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  event_id uuid NOT NULL REFERENCES events (id),
  tari_asset_id TEXT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  synced_at TIMESTAMP NULL,
  ticket_delta BIGINT NOT NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_ticket_allocations_event_id ON ticket_allocations (event_id);
