ALTER TABLE event_artists
    -- If the artist is the headliner this will be 0
    ADD importance INTEGER NOT NULL DEFAULT 1,
    ADD stage_id UUID NULL REFERENCES stages(id) ON DELETE CASCADE;