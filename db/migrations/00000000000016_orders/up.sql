
-- Define the orders table
CREATE TABLE orders (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id),
  status TEXT NOT NULL,
  order_type TEXT not null,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_orders_user_id ON orders (user_id);
