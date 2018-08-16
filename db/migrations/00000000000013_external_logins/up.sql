
CREATE TABLE external_logins (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  site TEXT NOT NULL,
  access_token TEXT NOT NULL,
  external_user_id TEXT NOT NULL
);

CREATE INDEX index_external_logins_user_id ON external_logins (user_id);
CREATE UNIQUE INDEX index_external_logins_external_user_id_site ON external_logins (external_user_id, site);
