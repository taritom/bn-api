CREATE TABLE slugs
(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    slug VARCHAR(255) NOT NULL,
    main_table TEXT NOT NULL,
    main_table_id uuid NOT NULL,
    slug_type TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_slugs_slug ON slugs (slug);
CREATE INDEX index_slugs_slug_type ON slugs (slug_type);
CREATE INDEX index_slugs_main_table_id_main_table ON slugs (main_table_id, main_table);

INSERT INTO slugs (main_table, main_table_id, slug, slug_type)
SELECT 'Events', e.id, e.slug, 'Event'
FROM events e;

ALTER TABLE events ADD slug_id UUID;
CREATE UNIQUE INDEX index_events_slug_id ON events (slug_id);

ALTER TABLE organizations ADD slug_id UUID;
CREATE UNIQUE INDEX index_organizations_slug_id ON organizations (slug_id);

ALTER TABLE venues ADD slug_id UUID;
CREATE UNIQUE INDEX index_venues_slug_id ON venues (slug_id);

UPDATE events
SET slug_id = s.id
FROM events e
JOIN slugs s ON e.slug = s.slug
WHERE e.id = events.id;

DROP INDEX IF EXISTS index_events_slug;
ALTER TABLE events DROP slug;

INSERT INTO slugs (main_table, main_table_id, slug, slug_type)
SELECT 'Venues', v.id, CONCAT(regexp_replace(regexp_replace(lower(trim(v.name)), '[\(\)]+', '', 'gi'), '[^a-z0-9\\-_]+', '-', 'gi'), '-', SUBSTR(MD5(RANDOM()::TEXT), 0, 6)), 'Venue'
FROM venues v;

INSERT INTO slugs (main_table, main_table_id, slug, slug_type)
SELECT 'Organizations', o.id, CONCAT(regexp_replace(regexp_replace(lower(trim(o.name)), '[\(\)]+', '', 'gi'), '[^a-z0-9\\-_]+', '-', 'gi'), '-', SUBSTR(MD5(RANDOM()::TEXT), 0, 6)), 'Organization'
FROM organizations o;

INSERT INTO slugs (main_table, main_table_id, slug, slug_type)
SELECT 'Venues', v.id, regexp_replace(regexp_replace(lower(trim(v.city)), '[\(\)]+', '', 'gi'), '[^a-z0-9\\-_]+', '-', 'gi'), 'City'
FROM venues v;

UPDATE organizations
SET slug_id = s.id
FROM slugs s
WHERE organizations.id = s.main_table_id
AND s.slug_type = 'Organization';

UPDATE venues
SET slug_id = s.id
FROM slugs s
WHERE venues.id = s.main_table_id
AND s.slug_type = 'Venue';
