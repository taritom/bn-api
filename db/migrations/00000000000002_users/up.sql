-- Define the users table
CREATE TABLE users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  first_name TEXT NOT NULL,
  last_name TEXT NOT NULL,
  email TEXT NULL UNIQUE,
  phone TEXT NULL,
  profile_pic_url TEXT NULL,
  thumb_profile_pic_url TEXT NULL,
  cover_photo_url TEXT NULL,
  hashed_pw TEXT NOT NULL,
  password_modified_at TIMESTAMP NOT NULL DEFAULT now(),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  last_used TIMESTAMP DEFAULT NULL,
  active BOOLEAN NOT NULL DEFAULT 't',
  role text[] NOT NULL,
  password_reset_token uuid NULL,
  password_reset_requested_at TIMESTAMP NULL,
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_users_uuid ON users (id);
CREATE INDEX index_users_email ON users (email);
CREATE INDEX index_users_password_reset_token ON users (password_reset_token);
