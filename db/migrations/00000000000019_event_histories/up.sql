-- Define the event_histories table
CREATE TABLE event_histories (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  event_id uuid NOT NULL REFERENCES events (id) ON DELETE CASCADE,
  order_id uuid NOT NULL REFERENCES orders (id) ON DELETE CASCADE,
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  protocol_reference_hash VARCHAR(255) NOT NULL
);

-- Indices
CREATE INDEX index_event_histories_event_id ON event_histories (event_id);
CREATE INDEX index_event_histories_order_id ON event_histories (order_id);
CREATE INDEX index_event_histories_user_id ON event_histories (user_id);
