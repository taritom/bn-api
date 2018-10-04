CREATE OR REPLACE FUNCTION ticket_pricing_no_overlapping_periods(UUID, UUID, TIMESTAMP, TIMESTAMP) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        SELECT NOT EXISTS (
            SELECT id
            FROM ticket_pricing
            WHERE
                ID <> $1
            AND
                ticket_type_id = $2
            AND
            (
                (start_date <= $3 AND end_date > $3)
            OR
                (start_date <= $4 AND end_date > $4)
            OR
                (start_date >= $3 AND end_date < $4)
            )
        )
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE ticket_pricing ADD CONSTRAINT ticket_pricing_start_date_prior_to_end_date CHECK (start_date < end_date);
ALTER TABLE ticket_pricing ADD CONSTRAINT ticket_pricing_no_overlapping_periods CHECK(ticket_pricing_no_overlapping_periods(id, ticket_type_id, start_date, end_date));
