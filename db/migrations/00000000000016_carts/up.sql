
CREATE TABLE carts (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id uuid NOT NULL REFERENCES users (id),
  created_at TIMESTAMP NOT NULL DEFAULT now()
  );

-- Indices
CREATE INDEX index_carts_user_id ON carts (user_id);
