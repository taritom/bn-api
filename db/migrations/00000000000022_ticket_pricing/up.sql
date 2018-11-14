CREATE TABLE ticket_pricing (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  ticket_type_id uuid NOT NULL REFERENCES ticket_types (id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  status TEXT NOT NULL,
  price_in_cents BIGINT NOT NULL,
  start_date TIMESTAMP NOT NULL CHECK (start_date < end_date),
  end_date TIMESTAMP NOT NULL,
  is_box_office_only BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT  NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_ticket_pricing_ticket_type_id ON ticket_pricing(ticket_type_id);
CREATE INDEX index_is_box_office_only ON ticket_pricing(is_box_office_only);