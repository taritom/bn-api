ALTER TABLE ticket_types ADD COLUMN end_date_type TEXT NULL;
UPDATE ticket_types SET end_date_type = 'Manual';
ALTER TABLE ticket_types ALTER COLUMN end_date_type SET NOT NULL;

ALTER TABLE ticket_types ALTER COLUMN end_date DROP NOT NULL;
