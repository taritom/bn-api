CREATE TABLE event_report_subscribers
(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    event_id UUID NOT NULL REFERENCES events(id),
    email TEXT NOT NULL,
    report_type TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX index_event_report_subscribers_report_type_event_id_email ON event_report_subscribers (report_type, event_id, email);
CREATE INDEX index_event_report_subscribers_email ON event_report_subscribers (email);
CREATE INDEX index_event_report_subscribers_event_id ON event_report_subscribers (event_id);
