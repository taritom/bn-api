CREATE TABLE ticket_type_codes
(
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  ticket_type_id uuid NOT NULL REFERENCES ticket_types (id) ON DELETE CASCADE,
  code_id uuid NOT NULL REFERENCES codes (id) ON DELETE CASCADE,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX index_ticket_type_codes_ticket_type_id_code_id ON ticket_type_codes (ticket_type_id, code_id);

CREATE OR REPLACE FUNCTION ticket_type_code_ticket_type_id_valid(UUID, UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        select exists (
            select * from ticket_types tt join events e on tt.event_id = e.id join codes d on d.event_id = e.id where tt.id = $2 and d.id = $1
        )
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE ticket_type_codes ADD CONSTRAINT ticket_type_code_ticket_type_id_valid CHECK(ticket_type_code_ticket_type_id_valid(code_id, ticket_type_id));
