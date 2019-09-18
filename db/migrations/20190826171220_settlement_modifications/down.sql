ALTER TABLE organizations
  DROP COLUMN settlement_type;
ALTER TABLE settlements
  ADD user_id Uuid NOT NULL;

DROP INDEX IF EXISTS index_orders_settlement_id;
ALTER TABLE orders
  DROP settlement_id;

DROP INDEX IF EXISTS index_settlement_entries_settlement_id;
DROP INDEX IF EXISTS index_settlement_entries_event_id;
DROP TABLE IF EXISTS settlement_entries;

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
