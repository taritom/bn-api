DROP INDEX IF EXISTS index_events_slug;

ALTER TABLE events
    DROP slug;
