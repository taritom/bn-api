ALTER TABLE organizations
    ALTER COLUMN event_fee_in_cents DROP NOT NULL;
ALTER TABLE organizations
    ALTER COLUMN event_fee_in_cents DROP DEFAULT;

ALTER TABLE events
    ALTER COLUMN fee_in_cents DROP NOT NULL;
ALTER TABLE events
    ALTER COLUMN fee_in_cents DROP DEFAULT;
