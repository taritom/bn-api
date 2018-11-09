CREATE TABLE codes
(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    name TEXT NOT NULL,
    event_id uuid NOT NULL REFERENCES events(id),
    code_type TEXT NOT NULL,
    redemption_code TEXT NOT NULL CHECK (length(redemption_code) >= 6),
    max_uses BIGINT NOT NULL CHECK (max_uses > 0),
    discount_in_cents bigint NULL CHECK (code_type <> 'Discount' or discount_in_cents > 0),
    start_date TIMESTAMP NOT NULL CHECK (start_date < end_date),
    end_date TIMESTAMP NOT NULL,
    max_tickets_per_user BIGINT NULL CHECK (coalesce (max_tickets_per_user, 1) >= 0),
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_codes_event_id_code_type ON codes (event_id, code_type);

ALTER TABLE codes ADD CONSTRAINT codes_start_date_prior_to_end_date CHECK (start_date < end_date);
CREATE UNIQUE INDEX index_codes_redemption_code ON codes(redemption_code);

ALTER TABLE order_items ADD code_id UUID NULL REFERENCES codes (id);
CREATE INDEX index_order_items_code_id ON order_items (code_id);

CREATE OR REPLACE FUNCTION order_items_code_id_max_uses_valid(UUID, UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
      (select max_uses from codes where id = $2) >
      (
          select count(orders.id)
          from orders
          join order_items
          on order_items.order_id = orders.id
          where orders.id <> $1 and order_items.code_id = $2
          group by orders.id
      )
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE order_items ADD CONSTRAINT order_items_code_id_max_uses_valid CHECK(order_items_code_id_max_uses_valid(order_id, code_id));

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
