CREATE TABLE genres
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid() NOT NULL,
    name              TEXT        NOT NULL,
    created_at        TIMESTAMP   NOT NULL DEFAULT now(),
    updated_at        TIMESTAMP   NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX index_genres_name ON genres (name);

INSERT INTO genres(name)
SELECT x
FROM unnest(ARRAY[
  'acoustic',
  'afrobeat',
  'alt-rock',
  'alternative',
  'ambient',
  'anime',
  'black-metal',
  'bluegrass',
  'blues',
  'bossanova',
  'brazil',
  'breakbeat',
  'british',
  'cantopop',
  'chicago-house',
  'children',
  'chill',
  'classical',
  'club',
  'comedy',
  'country',
  'dance',
  'dancehall',
  'death-metal',
  'deep-house',
  'detroit-techno',
  'disco',
  'disney',
  'drum-and-bass',
  'dub',
  'dubstep',
  'edm',
  'electro',
  'electronic',
  'emo',
  'folk',
  'forro',
  'french',
  'funk',
  'garage',
  'german',
  'gospel',
  'goth',
  'grindcore',
  'groove',
  'grunge',
  'guitar',
  'happy',
  'hard-rock',
  'hardcore',
  'hardstyle',
  'heavy-metal',
  'hip-hop',
  'holidays',
  'honky-tonk',
  'house',
  'idm',
  'indian',
  'indie',
  'indie-pop',
  'industrial',
  'iranian',
  'j-dance',
  'j-idol',
  'j-pop',
  'j-rock',
  'jazz',
  'k-pop',
  'kids',
  'latin',
  'latino',
  'malay',
  'mandopop',
  'metal',
  'metal-misc',
  'metalcore',
  'minimal-techno',
  'movies',
  'mpb',
  'new-age',
  'new-release',
  'opera',
  'pagode',
  'party',
  'philippines-opm',
  'piano',
  'pop',
  'pop-film',
  'post-dubstep',
  'power-pop',
  'progressive-house',
  'psych-rock',
  'punk',
  'punk-rock',
  'r-n-b',
  'rainy-day',
  'reggae',
  'reggaeton',
  'road-trip',
  'rock',
  'rock-n-roll',
  'rockabilly',
  'romance',
  'sad',
  'salsa',
  'samba',
  'sertanejo',
  'show-tunes',
  'singer-songwriter',
  'ska',
  'sleep',
  'songwriter',
  'soul',
  'soundtracks',
  'spanish',
  'study',
  'summer',
  'swedish',
  'synth-pop',
  'tango',
  'techno',
  'trance',
  'trip-hop',
  'turkish',
  'work-out',
  'world-music'
]) x;

CREATE TABLE event_genres
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid() NOT NULL,
    event_id          UUID        NOT NULL REFERENCES events(id) ON DELETE CASCADE,
    genre_id          UUID        NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    created_at        TIMESTAMP   NOT NULL DEFAULT now(),
    updated_at        TIMESTAMP   NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX index_event_genres_genre_id_event_id ON event_genres (genre_id, event_id);
CREATE INDEX index_event_genres_event_id ON event_genres (event_id);

CREATE TABLE artist_genres
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid() NOT NULL,
    artist_id         UUID        NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    genre_id          UUID        NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    created_at        TIMESTAMP   NOT NULL DEFAULT now(),
    updated_at        TIMESTAMP   NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX index_artist_genres_genre_id_artist_id ON artist_genres (genre_id, artist_id);
CREATE INDEX index_artist_genres_artist_id ON artist_genres (artist_id);

CREATE TABLE user_genres
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid() NOT NULL,
    user_id           UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    genre_id          UUID        NOT NULL REFERENCES genres(id) ON DELETE CASCADE,
    created_at        TIMESTAMP   NOT NULL DEFAULT now(),
    updated_at        TIMESTAMP   NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX index_user_genres_genre_id_user_id ON user_genres (genre_id, user_id);
CREATE INDEX index_user_genres_user_id ON user_genres (user_id);
