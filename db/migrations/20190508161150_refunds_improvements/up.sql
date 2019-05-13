CREATE TABLE refunds (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  order_id uuid NOT NULL REFERENCES orders(id),
  user_id uuid NOT NULL REFERENCES users(id),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX index_refunds_order_id ON refunds(order_id);
CREATE INDEX index_refunds_user_id ON refunds(user_id);

ALTER TABLE payments
    ADD COLUMN refund_id uuid NULL REFERENCES refunds(id);
CREATE INDEX index_payments_refund_id ON payments(refund_id);

CREATE TABLE refund_items (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  refund_id uuid NOT NULL REFERENCES refunds(id),
  order_item_id uuid NOT NULL REFERENCES order_items(id),
  quantity BIGINT NOT NULL,
  amount BIGINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);
CREATE INDEX index_refund_items_order_item_id ON refund_items(order_item_id);
CREATE INDEX index_refund_items_refund_id ON refund_items(refund_id);
