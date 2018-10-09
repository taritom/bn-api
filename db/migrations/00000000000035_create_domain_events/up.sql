CREATE TABLE domain_events (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    event_type TEXT NOT NULL,
    display_text TEXT NOT NULL,
    event_data json NULL,
    main_table TEXT NOT NULL,
    main_id uuid NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);
