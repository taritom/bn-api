ALTER TABLE users
  ADD last_cart_id UUID NULL REFERENCES orders (id);

CREATE INDEX index_users_last_cart_id
  ON users (last_cart_id);