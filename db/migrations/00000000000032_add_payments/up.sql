CREATE TABLE payments (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  order_id uuid NOT NULL REFERENCES orders(id),
  created_by uuid NOT NULL REFERENCES users(id),
  status TEXT NOT NULL,
  payment_method TEXT NOT NULL,
  amount BIGINT NOT NULL,
  provider TEXT NOT NULL,
  external_reference TEXT NOT NULL,
  raw_data TEXT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_payments_order_id ON payments (order_id);
CREATE INDEX index_payments_created_by ON payments(created_by);