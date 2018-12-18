ALTER TABLE organizations ADD client_event_fee_in_cents BIGINT NOT NULL DEFAULT 0;
ALTER TABLE organizations ADD company_event_fee_in_cents BIGINT NOT NULL DEFAULT 0;

ALTER TABLE events ADD client_fee_in_cents BIGINT NOT NULL DEFAULT 0;
ALTER TABLE events ADD company_fee_in_cents BIGINT NOT NULL DEFAULT 0;