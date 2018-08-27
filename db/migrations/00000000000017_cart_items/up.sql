
CREATE TABLE cart_items (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  cart_id uuid NOT NULL REFERENCES carts (id),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  ticket_allocation_id uuid NOT NULL REFERENCES ticket_allocations(id),
  quantity bigint not null
  );

-- Indices
CREATE INDEX index_cart_items_cart_id ON cart_items (cart_id);
CREATE INDEX index_cart_items_ticket_allocation_id ON cart_items (ticket_allocation_id);
