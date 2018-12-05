ALTER TABLE order_items
  ADD company_fee_in_cents bigint NOT NULL,
  ADD client_fee_in_cents bigint NOT NULL;
