CREATE TABLE settlement_adjustments
(
    id                            UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    settlement_id                 UUID NOT NULL REFERENCES settlements (id) ON DELETE CASCADE,
    amount_in_cents               BIGINT NOT NULL,
    note                          TEXT NULL,
    settlement_adjustment_type    TEXT NOT NULL,
    created_at                    TIMESTAMP NOT NULL DEFAULT now(),
    updated_at                    TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX index_settlement_adjustments_settlement_id ON settlement_adjustments (settlement_id);
