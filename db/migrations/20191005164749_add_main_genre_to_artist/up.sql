ALTER TABLE artists
    ADD COLUMN IF NOT EXISTS main_genre_id UUID NULL REFERENCES genres (id);

