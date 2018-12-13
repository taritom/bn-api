ALTER TABLE orders ADD paid_at timestamp NULL;

UPDATE orders SET paid_at = order_date where status = 'Paid';
