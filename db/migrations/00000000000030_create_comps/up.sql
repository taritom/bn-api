CREATE TABLE comps
(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    name TEXT NOT NULL,
    phone TEXT NULL,
    email TEXT NULL,
    hold_id uuid NOT NULL REFERENCES holds(id) ON DELETE CASCADE,
    quantity INT NOT NULL CHECK (quantity >= 0),
    redemption_code TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX index_comps_redemption_code ON holds(redemption_code);
CREATE UNIQUE INDEX index_comps_hold_id_name ON comps (
	hold_id,
	name
);

CREATE OR REPLACE FUNCTION comps_quantity_valid_for_hold_quantity(UUID, UUID, INT) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        select (
          (select coalesce(sum(quantity), 0) + $3 from comps where hold_id = $1 and id <> $2) <=
          (select count(*) from ticket_instances where hold_id = $1)
        )
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE comps ADD CONSTRAINT comps_quantity_valid_for_hold_quantity CHECK(comps_quantity_valid_for_hold_quantity(hold_id, id, quantity));

CREATE OR REPLACE FUNCTION comps_hold_type_valid_for_comp_creation(UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        select hold_type = 'Comp' from holds where id = $1
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE comps ADD CONSTRAINT comps_hold_type_valid_for_comp_creation CHECK(comps_hold_type_valid_for_comp_creation(hold_id));
