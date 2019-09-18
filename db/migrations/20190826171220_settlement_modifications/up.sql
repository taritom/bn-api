ALTER TABLE organizations
   ADD settlement_type TEXT NOT NULL DEFAULT 'PostEvent';
ALTER TABLE settlements
   DROP COLUMN user_id;

DROP INDEX IF EXISTS index_settlement_transactions_settlement_id;
DROP INDEX IF EXISTS index_settlement_transactions_event_id;
DROP TABLE IF EXISTS settlement_transactions;

CREATE TABLE settlement_entries
(
    id                            UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    settlement_id                 UUID NOT NULL REFERENCES settlements (id) ON DELETE CASCADE,
    event_id                      UUID NOT NULL REFERENCES events (id),
    ticket_type_id                UUID NULL REFERENCES ticket_types (id),
    face_value_in_cents           BIGINT NOT NULL,
    revenue_share_value_in_cents  BIGINT NOT NULL,
    online_sold_quantity          BIGINT NOT NULL,
    fee_sold_quantity             BIGINT NOT NULL,
    total_sales_in_cents          BIGINT NOT NULL,
    settlement_entry_type         TEXT NOT NULL,
    created_at                    TIMESTAMP NOT NULL DEFAULT now(),
    updated_at                    TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX index_settlement_entries_settlement_id ON settlement_entries (settlement_id);
CREATE INDEX index_settlement_entries_event_id ON settlement_entries (event_id);

ALTER TABLE orders
  ADD settlement_id Uuid NULL references settlements(id);
CREATE INDEX index_orders_settlement_id ON orders (settlement_id);
