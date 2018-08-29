DROP INDEX IF EXISTS index_cart_items_ticket_allocation_id;

alter table cart_items
  drop column ticket_allocation_id;

alter table cart_items
  add price_point_id Uuid Not null REFERENCES price_points (id);

CREATE INDEX index_cart_items_price_point_id ON cart_items (price_point_id);