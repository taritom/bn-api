DROP INDEX index_order_items_ticket_type_id;

ALTER TABLE order_items
  DROP COLUMN ticket_type_id;

ALTER TABLE order_items
  DROP COLUMN quantity;

ALTER TABLE order_items
  ADD ticket_instance_id Uuid NULL REFERENCES ticket_instances (id);

ALTER TABLE order_items
  ADD ticket_pricing_id Uuid NULL REFERENCES ticket_pricing(id);

ALTER TABLE order_items
  ADD fee_schedule_range_id Uuid NULL REFERENCES fee_schedule_ranges(id);

ALTER TABLE order_items
  ADD parent_id Uuid NULL REFERENCES order_items(id);

-- Indices
CREATE INDEX index_order_items_ticket_instance_id ON order_items(ticket_instance_id);
CREATE INDEX index_order_items_ticket_pricing_id ON order_items(ticket_pricing_id);
CREATE INDEX index_order_items_fee_schedule_range_id ON order_items(fee_schedule_range_id);
CREATE INDEX index_order_items_parent_id ON order_items(parent_id);

