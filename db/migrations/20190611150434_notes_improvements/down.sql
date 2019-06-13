ALTER TABLE orders
    ADD note TEXT NULL;

UPDATE orders
SET note = n.note
FROM (
  SELECT DISTINCT ON (order_id) order_id, note
  FROM notes
  WHERE main_table = 'Orders'
  ORDER BY order_id, created_at DESC
) n
WHERE n.order_id = orders.id;

DROP INDEX IF EXISTS index_notes_main_table_main_id;
DROP INDEX IF EXISTS index_notes_main_id;
DROP INDEX IF EXISTS index_notes_user_id;
DROP TABLE IF EXISTS notes;
