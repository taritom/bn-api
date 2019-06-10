ALTER TABLE events
    ADD slug VARCHAR(255);

CREATE UNIQUE INDEX index_events_slug ON events (slug);

UPDATE events
SET slug = CONCAT(SUBSTR(LOWER(REPLACE(name, ' ', '-')), 0, 249), '-', SUBSTR(MD5(RANDOM()::TEXT), 0, 6));

ALTER TABLE events
    ALTER slug SET NOT NULL;
