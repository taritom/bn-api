CREATE OR REPLACE FUNCTION order_items_code_id_max_tickets_per_user_valid(UUID, UUID, UUID, bigint) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
      (
        -- Use max bigint value if null
        select coalesce(max_tickets_per_user, (-(((2^(8*pg_column_size(1::bigint)-2))::bigint << 1)+1)))
        from codes where id = $3
      )
      >=
      (
        select coalesce(sum(quantity), 0) + $4
        from order_items
        join orders
        on orders.id = order_items.order_id
        where order_items.id <> $1
        and code_id = $3
        and user_id = (select user_id from orders where id = $2)
      )
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE order_items ADD CONSTRAINT order_items_code_id_max_tickets_per_user_valid CHECK(order_items_code_id_max_tickets_per_user_valid(id, order_id, code_id, quantity));
