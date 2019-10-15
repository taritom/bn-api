ALTER TABLE events
    ADD slug VARCHAR(255);

CREATE UNIQUE INDEX index_events_slug ON events (slug);

UPDATE events
SET slug = s.slug
FROM slugs s
WHERE s.id = events.slug_id;

ALTER TABLE events
    ALTER slug SET NOT NULL;

DROP INDEX IF EXISTS index_organizations_slug_id;
ALTER TABLE organizations DROP slug_id;

DROP INDEX IF EXISTS index_venues_slug_id;
ALTER TABLE venues DROP slug_id;

DROP INDEX IF EXISTS index_events_slug_id;
ALTER TABLE events DROP slug_id;

DROP INDEX IF EXISTS index_slugs_main_table_id_main_table;
DROP INDEX IF EXISTS index_slugs_slug_type;
DROP INDEX IF EXISTS index_slugs_slug;
DROP TABLE IF EXISTS slugs;
