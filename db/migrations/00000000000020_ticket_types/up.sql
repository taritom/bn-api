CREATE TABLE ticket_types (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  event_id uuid NOT NULL REFERENCES events(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  status TEXT NOT NULL,
  start_date TIMESTAMP NOT NULL CHECK (start_date < end_date),
  end_date TIMESTAMP NOT NULL,
  increment INT NOT NULL DEFAULT 1,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_ticket_types_event_id ON ticket_types (event_id);
CREATE UNIQUE INDEX index_ticket_types_event_id_name on ticket_types (event_id, name);
