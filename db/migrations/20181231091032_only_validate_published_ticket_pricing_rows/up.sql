CREATE OR REPLACE FUNCTION ticket_pricing_no_overlapping_periods(UUID, UUID, TIMESTAMP, TIMESTAMP, BOOLEAN, BOOLEAN) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        -- $5 = is_box_office_only
        -- $6 = status == Default
        SELECT $5 OR $6 OR NOT EXISTS (
            SELECT id
            FROM ticket_pricing
            WHERE
                -- Filter out current record being updated
                ID <> $1
            AND
                -- Filter out is_box_office_only prices they can overlap dates
                is_box_office_only = FALSE
            AND
                -- Only compare against Published price points (Not Deleted or Default)
                status = 'Published'
            AND
                -- Filter to the current ticket type
                ticket_type_id = $2
            AND
            (
                -- Does any period overlap the start date
                (start_date <= $3 AND end_date > $3)
            OR
                -- Does any period overlap the end date
                (start_date < $4 AND end_date >= $4)
            OR
                -- Does this period completely overlap another period
                (start_date >= $3 AND end_date <= $4)
            )
        )
    );
END $$ LANGUAGE 'plpgsql';
