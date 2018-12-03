CREATE TABLE domain_actions (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    domain_event_id uuid NULL references domain_events(id),
    domain_action_type TEXT NOT NULL,
    communication_channel_type TEXT NULL,
    payload json NOT NULL,
    main_table TEXT NOT NULL,
    main_table_id uuid NOT NULL,
    scheduled_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    last_attempted_at TIMESTAMP NULL,
    attempt_count bigint NOT NULL,
    max_attempt_count bigint NOT NULL,
    status TEXT NOT NULL,
    last_failure_reason TEXT NULL,
    blocked_until TIMESTAMP NOT NULL DEFAULT now(),
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE INDEX index_domain_actions_id ON domain_actions(id);
CREATE INDEX index_domain_actions_domain_event_id ON domain_actions(domain_event_id);
CREATE INDEX index_domain_actions_main_table_id ON domain_actions(main_table_id);
