DROP INDEX IF EXISTS index_event_users_event_id_user_id;
DROP TABLE IF EXISTS event_users;

ALTER TABLE organization_users
   ADD event_ids uuid[] NOT NULL DEFAULT '{}';
