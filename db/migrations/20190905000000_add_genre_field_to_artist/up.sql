alter table artists
    add genres_id UUID NULL REFERENCES genres(id);
