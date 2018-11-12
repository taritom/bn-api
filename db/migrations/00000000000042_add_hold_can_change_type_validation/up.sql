CREATE OR REPLACE FUNCTION hold_can_change_type(UUID, Text) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        select case when
          $2 = 'Comp'
        then
          't'
        else
          not exists(
            select oi.id
            from order_items oi
            join comps c
            on c.id = oi.comp_id
            where c.hold_id = $1
          )
        end
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE holds ADD CONSTRAINT hold_can_change_type CHECK(hold_can_change_type(id, hold_type));
