ALTER TABLE organizations DROP COLUMN client_event_fee_in_cents;
ALTER TABLE organizations DROP COLUMN company_event_fee_in_cents;

ALTER TABLE events DROP COLUMN client_fee_in_cents;
ALTER TABLE events DROP COLUMN company_fee_in_cents;