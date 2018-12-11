UPDATE organizations
SET event_fee_in_cents = 0 where event_fee_in_cents = NULL;

UPDATE events
SET fee_in_cents = 0 where fee_in_cents = NULL;

ALTER TABLE organizations
    ALTER COLUMN event_fee_in_cents SET NOT NULL;
ALTER TABLE organizations
    ALTER COLUMN event_fee_in_cents SET DEFAULT 0;

ALTER TABLE events
    ALTER COLUMN fee_in_cents SET NOT NULL;
ALTER TABLE events
    ALTER COLUMN fee_in_cents SET DEFAULT 0;
