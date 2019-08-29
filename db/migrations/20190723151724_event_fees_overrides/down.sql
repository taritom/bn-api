ALTER TABLE events
    ALTER COLUMN client_fee_in_cents SET DEFAULT 0;

ALTER TABLE events
    ALTER COLUMN company_fee_in_cents SET DEFAULT 0;

ALTER TABLE events
    ADD COLUMN fee_in_cents BIGINT;


UPDATE events
SET client_fee_in_cents = o.client_event_fee_in_cents,
company_fee_in_cents = o.company_event_fee_in_cents,
fee_in_cents = o.company_event_fee_in_cents + o.client_event_fee_in_cents
FROM organizations o
WHERE o.id = events.organization_id;

ALTER TABLE events
    ALTER COLUMN client_fee_in_cents SET NOT NULL;
ALTER TABLE events
    ALTER COLUMN company_fee_in_cents SET NOT NULL;
ALTER TABLE events
    ALTER COLUMN fee_in_cents SET NOT NULL;
