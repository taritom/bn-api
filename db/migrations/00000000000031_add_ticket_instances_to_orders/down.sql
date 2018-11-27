DROP INDEX IF EXISTS index_order_items_ticket_pricing_id;
DROP INDEX IF EXISTS index_order_items_fee_schedule_range_id;
DROP INDEX IF EXISTS index_order_items_parent_id;
DROP INDEX IF EXISTS index_order_items_hold_id;


ALTER TABLE order_items
 DROP COLUMN ticket_pricing_id;

ALTER TABLE order_items
  DROP COLUMN fee_schedule_range_id;

ALTER TABLE order_items
  DROP COLUMN parent_id;
