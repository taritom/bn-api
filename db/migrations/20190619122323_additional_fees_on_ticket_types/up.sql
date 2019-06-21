ALTER TABLE ticket_types
    ADD additional_fee_in_cents BIGINT NOT NULL DEFAULT (0);
ALTER TABLE organizations
    ADD max_additional_fee_in_cents BIGINT NOT NULL DEFAULT (0);
