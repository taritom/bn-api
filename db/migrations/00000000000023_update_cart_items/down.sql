

DROP INDEX IF EXISTS index_cart_items_price_point_id;


alter table cart_items
  drop column price_point_id;

alter table cart_items
  add ticket_allocation_id uuid NOT NULL REFERENCES ticket_allocations(id);

CREATE INDEX index_cart_items_ticket_allocation_id ON cart_items (ticket_allocation_id);
