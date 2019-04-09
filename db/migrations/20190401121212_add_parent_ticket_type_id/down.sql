ALTER TABLE ticket_types
    DROP
        CONSTRAINT check_ticket_types_start_date_parent_id;


ALTER TABLE ticket_types
    DROP rank;

DROP INDEX index_ticket_type_parent_id;


ALTER TABLE ticket_types
    DROP parent_id;

ALTER TABLE ticket_types
    ALTER COLUMN start_date SET NOT NULL;


ALTER TABLE ticket_pricing
    DROP CONSTRAINT ticket_pricing_check;

ALTER TABLE ticket_pricing
    ADD CONSTRAINT ticket_pricing_check
        CHECK (start_date <= end_date);