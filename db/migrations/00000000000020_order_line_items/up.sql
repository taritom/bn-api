-- Define the order line items table
CREATE TABLE order_line_items (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  order_id uuid NOT NULL REFERENCES orders (id) ON DELETE CASCADE
);

-- Indices
CREATE INDEX index_order_line_items_order_id ON order_line_items (order_id);
