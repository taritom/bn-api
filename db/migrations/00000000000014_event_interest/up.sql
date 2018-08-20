-- Define the event_interest table
CREATE TABLE event_interest(
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  event_id uuid NOT NULL REFERENCES events (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE
);

-- Indices
CREATE UNIQUE INDEX index_event_interest_event_id_user_id ON event_interest (event_id,user_id);
