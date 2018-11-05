CREATE OR REPLACE FUNCTION redemption_code_unique_per_event(UUID, TEXT, TEXT) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        select not exists (
          select redemption_code from codes where (id <> $1 or $2 <> 'codes') and redemption_code = $3 union
          select redemption_code from holds where (id <> $1 or $2 <> 'holds') and redemption_code = $3
        )
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE codes ADD CONSTRAINT redemption_code_unique_per_event CHECK(redemption_code_unique_per_event(id, 'codes', redemption_code));
ALTER TABLE holds ADD CONSTRAINT redemption_code_unique_per_event CHECK(redemption_code_unique_per_event(id, 'holds', redemption_code));
