CREATE TABLE notes
(
    id UUID PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    note TEXT NOT NULL,
    main_table TEXT NOT NULL,
    main_id uuid NOT NULL,
    deleted_by UUID NULL REFERENCES users(id),
    deleted_at TIMESTAMP NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

INSERT INTO notes (created_by, main_table, main_id, note, created_at, updated_at)
SELECT user_id, 'Orders', main_id, trim('\"' FROM (event_data->'note')::text) as note, created_at, created_at
FROM domain_events
WHERE main_table = 'Orders'
  AND event_type = 'OrderUpdated'
  AND event_data->'note' IS NOT NULL;

ALTER TABLE orders
    DROP COLUMN note;

CREATE INDEX index_notes_created_by ON notes (created_by);
CREATE INDEX index_notes_deleted_by ON notes (deleted_by);
CREATE INDEX index_notes_main_table_main_id ON notes (main_table, main_id);
CREATE INDEX index_notes_main_id ON notes (main_id);
