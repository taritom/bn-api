ALTER TABLE ticket_types DROP end_date_type;
ALTER TABLE ticket_types ALTER COLUMN end_date SET NOT NULL;
