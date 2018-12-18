ALTER TABLE organizations DROP COLUMN client_event_fee_in_cents;
ALTER TABLE organizations DROP COLUMN company_event_fee_in_cents;

ALTER TABLE events ADD client_fee_in_cents BIGINT NULL;
ALTER TABLE events ADD company_fee_in_cents BIGINT NULL;