DROP INDEX IF EXISTS index_user_genres_user_id;
DROP INDEX IF EXISTS index_user_genres_genre_id_user_id;
DROP TABLE IF EXISTS user_genres;

DROP INDEX IF EXISTS index_artist_genres_artist_id;
DROP INDEX IF EXISTS index_artist_genres_genre_id_artist_id;
DROP TABLE IF EXISTS artist_genres;

DROP INDEX IF EXISTS index_event_genres_event_id;
DROP INDEX IF EXISTS index_event_genres_genre_id_event_id;
DROP TABLE IF EXISTS event_genres;

DROP INDEX IF EXISTS index_genres_name;
DROP TABLE IF EXISTS genres;
