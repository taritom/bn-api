CREATE OR REPLACE FUNCTION redemption_code_unique_per_event(hold_id UUID, select_type TEXT, r_code TEXT, e_id UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        SELECT NOT exists (
            SELECT redemption_code, deleted_at
            FROM codes
            WHERE ((id <> $1 AND $2 = 'codes') OR $2 <> 'codes') AND redemption_code = $3 AND deleted_at IS NULL AND event_id = $4
            UNION SELECT redemption_code, deleted_at
            FROM holds
            WHERE ((id <> $1 AND $2 = 'holds') OR $2 <> 'holds') AND redemption_code = $3 AND deleted_at IS NULL AND event_id = $4

            )
    );
END $$ LANGUAGE 'plpgsql';