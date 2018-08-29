CREATE TABLE price_points (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  ticket_type_id uuid NOT NULL REFERENCES ticket_types (id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  status TEXT NOT NULL,
  price_in_cents BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_price_points_ticket_type_id ON price_points (ticket_type_id);
CREATE UNIQUE INDEX index_price_points_ticket_type_id_name ON price_points(ticket_type_id, name)

