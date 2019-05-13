DROP INDEX IF EXISTS index_refund_items_refund_id;
DROP INDEX IF EXISTS index_refund_items_order_item_id;
DROP TABLE IF EXISTS refund_items;

DROP INDEX IF EXISTS index_payments_refund_id;
ALTER TABLE payments
  DROP refund_id;

DROP INDEX IF EXISTS index_refunds_user_id;
DROP INDEX IF EXISTS index_refunds_order_id;
DROP TABLE IF EXISTS refunds;
