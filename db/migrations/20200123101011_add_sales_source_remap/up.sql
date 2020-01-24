CREATE TABLE source_aliases (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    from_source TEXT                                       NOT NULL,
    from_medium TEXT                                       NOT NULL,
    to_source   TEXT                                       NOT NULL,
    to_medium   TEXT                                       NOT NULL
)
