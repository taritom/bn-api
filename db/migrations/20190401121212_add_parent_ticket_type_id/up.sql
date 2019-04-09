ALTER TABLE ticket_types
    ADD parent_id UUID REFERENCES ticket_types (id);

CREATE INDEX index_ticket_type_parent_id ON ticket_types (parent_id);

ALTER TABLE ticket_types
    ALTER COLUMN start_date DROP NOT NULL;

ALTER TABLE ticket_types
    ADD rank INT NOT NULL DEFAULT (0);

ALTER TABLE ticket_types
    ADD CONSTRAINT check_ticket_types_start_date_parent_id CHECK ((parent_id IS NOT NULL AND start_date IS NULL) OR
                                                                  (parent_id IS NULL AND start_date IS NOT NULL) );



ALTER TABLE ticket_pricing
    DROP CONSTRAINT ticket_pricing_check;

ALTER TABLE ticket_pricing
    ADD CONSTRAINT ticket_pricing_check
        CHECK (start_date <= end_date);