-- Define the event_artists table
CREATE TABLE event_artists (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  event_id uuid NOT NULL REFERENCES events (id) ON DELETE CASCADE,
  artist_id uuid NOT NULL REFERENCES artists (id) ON DELETE CASCADE,
  rank INTEGER NOT NULL
);

-- Indices
CREATE INDEX index_event_artists_event_id ON event_artists (event_id);
CREATE INDEX index_event_artists_artist_id ON event_artists (artist_id);
CREATE UNIQUE INDEX index_event_artists_event_id_artist_id ON event_artists (event_id, artist_id);
