CREATE TABLE broadcasts
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid() NOT NULL,
    event_id          UUID        NOT NULL REFERENCES events (id),
    notification_type VARCHAR(20) NOT NULL,
    channel           VARCHAR(20) NOT NULL DEFAULT 'PushNotification',
    name              TEXT        NOT NULL,
    message           TEXT        NULL,
    send_at           TIMESTAMP   NULL,
    status            VARCHAR(20) NOT NULL DEFAULT 'Pending',
    progress          INTEGER     NOT NULL DEFAULT 0,
    created_at        TIMESTAMP   NOT NULL DEFAULT now(),
    updated_at        TIMESTAMP   NOT NULL DEFAULT now()
);
CREATE INDEX index_broadcasts_event_id ON broadcasts (event_id);
CREATE UNIQUE INDEX index_last_call_unique_per_event ON broadcasts (event_id) WHERE notification_type = 'LastCall' AND status != 'Cancelled';