-- Define the orders table
CREATE TABLE orders (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  event_id uuid NOT NULL REFERENCES events (id) ON DELETE CASCADE
);

-- Indices
CREATE INDEX index_orders_user_id ON orders (user_id);
CREATE INDEX index_orders_event_id ON orders (event_id);
