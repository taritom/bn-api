CREATE TABLE settlements
(
    id                   UUID PRIMARY KEY   DEFAULT gen_random_uuid() NOT NULL,
    organization_id      UUID      NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    user_id              uuid      NOT NULL REFERENCES users (id),
    start_time           TIMESTAMP NOT NULL,
    end_time             TIMESTAMP NOT NULL,
    status               TEXT      NOT NULL,
    comment              TEXT      NULL,
    only_finished_events BOOLEAN   NOT NULL DEFAULT TRUE,
    created_at           TIMESTAMP NOT NULL DEFAULT now(),
    updated_at           TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_settlements_organization_id ON settlements (organization_id);

CREATE TABLE settlement_transactions
(
    id                UUID PRIMARY KEY   DEFAULT gen_random_uuid() NOT NULL,
    settlement_id     UUID      NOT NULL REFERENCES settlements (id) ON DELETE CASCADE,
    event_id          UUID      NOT NULL REFERENCES events (id),
    order_item_id     UUID      NULL,
    settlement_status TEXT      NOT NULL DEFAULT 'PendingSettlement',
    transaction_type  TEXT      NOT NULL DEFAULT 'Manual',
    value_in_cents    BIGINT    NOT NULL,
    comment           TEXT      NULL,
    created_at        TIMESTAMP NOT NULL DEFAULT now(),
    updated_at        TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_settlement_transactions_settlement_id ON settlement_transactions (settlement_id);
CREATE INDEX index_settlement_transactions_event_id ON settlement_transactions (event_id);