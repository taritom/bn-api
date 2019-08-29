ALTER TABLE events
    DROP COLUMN fee_in_cents;

ALTER TABLE events
    ALTER COLUMN client_fee_in_cents DROP NOT NULL;
ALTER TABLE events
    ALTER COLUMN client_fee_in_cents DROP DEFAULT;

ALTER TABLE events
    ALTER COLUMN company_fee_in_cents DROP NOT NULL;
ALTER TABLE events
    ALTER COLUMN company_fee_in_cents DROP DEFAULT;
