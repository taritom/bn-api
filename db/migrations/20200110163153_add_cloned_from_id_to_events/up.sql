ALTER TABLE events
    ADD cloned_from_event_id Uuid NULL REFERENCES events(id);

CREATE INDEX index_events_cloned_from_event_id ON events (cloned_from_event_id);
