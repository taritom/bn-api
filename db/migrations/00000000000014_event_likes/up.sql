-- Define the event_likes table
CREATE TABLE event_likes (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  event_id uuid NOT NULL REFERENCES events (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE
);

-- Indices
CREATE UNIQUE INDEX index_event_likes_event_id_user_id ON event_likes (event_id,user_id);
