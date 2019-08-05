DROP TABLE IF EXISTS temporary_user_links;
DROP TABLE IF EXISTS temporary_users;

ALTER TABLE transfers
  DROP COLUMN direct;

ALTER TABLE transfers
  DROP COLUMN destination_temporary_user_id;

ALTER TABLE domain_events
  ADD COLUMN published_at TIMESTAMP NULL;

DROP TABLE IF EXISTS domain_event_published;
DROP TABLE IF EXISTS domain_event_publishers;
