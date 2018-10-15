CREATE OR REPLACE FUNCTION order_items_quantity_in_increments(TEXT, BIGINT, UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        SELECT
            case when $1 = 'Tickets' then ($2 % tt.increment) = 0 else 't' end
        FROM
            ticket_pricing tp
        JOIN
            ticket_types tt
        ON
            tp.ticket_type_id = tt.id
        WHERE
            tp.id = $3
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE order_items ADD CONSTRAINT order_items_quantity_in_increments CHECK(order_items_quantity_in_increments(item_type, quantity, ticket_pricing_id));
