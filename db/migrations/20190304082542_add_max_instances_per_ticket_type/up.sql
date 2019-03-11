ALTER TABLE organizations
    ADD COLUMN max_instances_per_ticket_type BIGINT NOT NULL DEFAULT 10000;
