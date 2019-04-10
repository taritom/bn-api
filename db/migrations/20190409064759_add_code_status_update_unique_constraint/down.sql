ALTER TABLE codes
    DROP COLUMN deleted_at;

ALTER TABLE holds
    ADD status TEXT NOT NULL DEFAULT 'Published';

UPDATE holds
    SET status = 'Deleted'
    WHERE deleted_at IS NOT NULL;

ALTER TABLE holds
  DROP COLUMN deleted_at;

DROP INDEX index_codes_redemption_code;
DROP INDEX index_holds_redemption_code;

CREATE OR REPLACE FUNCTION redemption_code_unique_per_event(UUID, TEXT, TEXT) RETURNS BOOLEAN AS $$
BEGIN
    RETURN (
        SELECT NOT exists (
          SELECT redemption_code
          FROM codes
          WHERE ((id <> $1 AND $2 = 'codes') OR $2 <> 'codes') AND redemption_code = $3
          UNION SELECT redemption_code
          FROM holds
          WHERE ((id <> $1 AND $2 = 'holds') OR $2 <> 'holds') AND redemption_code = $3

        )
    );
END $$ LANGUAGE 'plpgsql';

ALTER TABLE codes ADD CONSTRAINT redemption_code_unique_per_event CHECK(redemption_code_unique_per_event(id, 'codes', redemption_code));
ALTER TABLE holds ADD CONSTRAINT redemption_code_unique_per_event CHECK(redemption_code_unique_per_event(id, 'holds', redemption_code));