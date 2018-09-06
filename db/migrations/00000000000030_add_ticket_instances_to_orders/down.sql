
DROP INDEX IF EXISTS index_order_items_ticket_instance_id;
DROP INDEX IF EXISTS index_order_items_price_point_id;
DROP INDEX IF EXISTS index_order_items_fee_schedule_range_id;
DROP INDEX IF EXISTS index_order_items_parent_id;


ALTER TABLE order_items
 DROP COLUMN ticket_instance_id;

ALTER TABLE order_items
 DROP COLUMN price_point_id;

ALTER TABLE order_items
  DROP COLUMN fee_schedule_range_id;

ALTER TABLE order_items
  DROP COLUMN parent_id;

ALTER TABLE order_items
  ADD ticket_type_id Uuid NULL REFERENCES ticket_types(id);

ALTER TABLE order_items
  ADD quantity bigint NOT NULL;




CREATE INDEX index_order_items_ticket_type_id ON order_items (ticket_type_id);
