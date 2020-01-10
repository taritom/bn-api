DROP INDEX IF EXISTS index_events_cloned_from_event_id;
ALTER TABLE events
    DROP cloned_from_event_id;
