-- Define the push_notification_tokens table
CREATE TABLE push_notification_tokens (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id),
  token_source TEXT NOT NULL,
  token TEXT NOT NULL,
  last_notification_at TIMESTAMP NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_push_notification_tokens_user_id ON push_notification_tokens (user_id);