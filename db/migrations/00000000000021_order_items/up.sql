CREATE TABLE order_items (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  order_id uuid NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
  item_type text not null,
  ticket_type_id Uuid not null references ticket_types,
  quantity bigint not null,
  unit_price_in_cents bigint not null,
  created_at TIMESTAMP not null default now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()

);

-- Indices
CREATE INDEX index_order_items_order_id ON order_items (order_id);
CREATE INDEX index_order_items_ticket_type_id ON order_items (ticket_type_id);
