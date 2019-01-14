ALTER TABLE order_items DROP COLUMN refunded_quantity;

DROP INDEX IF EXISTS index_refunded_tickets_ticket_instance_id;
DROP INDEX IF EXISTS index_refunded_tickets_order_item_id;
DROP TABLE refunded_tickets;
