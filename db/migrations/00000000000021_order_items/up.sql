CREATE TABLE order_items (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  order_id uuid NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
  item_type text not null,
  ticket_type_id Uuid null references ticket_types,
  event_id uuid null references events,
  quantity bigint not null,
  unit_price_in_cents bigint not null,
  company_fee_in_cents bigint not null,
  client_fee_in_cents bigint not null,
  created_at TIMESTAMP not null default now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()

);

ALTER TABLE order_items
    add CONSTRAINT constraint_order_items_ticket_type_id CHECK (NOT (ticket_type_id is null and item_type = 'Tickets'));

-- Indices
CREATE INDEX index_order_items_order_id ON order_items (order_id);
CREATE INDEX index_order_items_ticket_type_id ON order_items (ticket_type_id);
CREATE INDEX index_order_items_event_id ON order_items (event_id);
