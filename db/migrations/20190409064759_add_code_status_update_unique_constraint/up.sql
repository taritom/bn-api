ALTER TABLE codes
    ADD deleted_at TIMESTAMP NULL;

ALTER TABLE holds DROP CONSTRAINT redemption_code_unique_per_event;
ALTER TABLE codes DROP CONSTRAINT redemption_code_unique_per_event;
DROP FUNCTION IF EXISTS redemption_code_unique_per_event(UUID, TEXT, TEXT);

ALTER TABLE holds
  ADD deleted_at TIMESTAMP NULL;

UPDATE holds
  SET deleted_at = updated_at
  WHERE status = 'Deleted';

ALTER TABLE holds
  DROP COLUMN status;

DROP INDEX index_codes_redemption_code;
CREATE UNIQUE INDEX index_codes_redemption_code ON codes(redemption_code, event_id) WHERE deleted_at IS NULL;

DROP INDEX index_holds_redemption_code;
CREATE UNIQUE INDEX index_holds_redemption_code ON holds(redemption_code, event_id) WHERE deleted_at IS NULL;

