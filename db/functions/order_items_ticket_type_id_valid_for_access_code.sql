CREATE OR REPLACE FUNCTION order_items_ticket_type_id_valid_for_access_code(UUID, UUID) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        -- If there's no access code associated or the code passed in is valid
        SELECT ttc.ticket_type_id is null or ttc2.id is not null
        FROM ticket_types tt
                 LEFT JOIN (
            -- Associated codes which grant access
            SELECT ttc.ticket_type_id
            FROM ticket_type_codes ttc
                     JOIN codes c ON ttc.code_id = c.id
                     JOIN ticket_types tt ON tt.id = ttc.ticket_type_id
            WHERE c.code_type = 'Access' AND tt.id = $1 AND c.deleted_at IS NULL
            GROUP BY ttc.ticket_type_id
        ) ttc ON ttc.ticket_type_id = tt.id
                 LEFT JOIN ticket_type_codes ttc2
                           ON ttc2.ticket_type_id = tt.id
                               AND ttc2.code_id = $2
        WHERE tt.id = $1
    );
END $$ LANGUAGE 'plpgsql';
