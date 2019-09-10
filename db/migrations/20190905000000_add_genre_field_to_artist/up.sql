alter table artists
    add main_genre_id UUID NULL REFERENCES genres(id);
