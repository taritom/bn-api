DROP INDEX IF EXISTS index_ticket_types_asset_id;
ALTER TABLE ticket_types
  DROP COLUMN asset_id;