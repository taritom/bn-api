CREATE TABLE organization_interactions
(
    id                UUID PRIMARY KEY     DEFAULT gen_random_uuid() NOT NULL,
    organization_id   UUID        NOT NULL REFERENCES organizations(id),
    user_id           UUID        NOT NULL REFERENCES users(id),
    first_interaction TIMESTAMP   NOT NULL DEFAULT now(),
    last_interaction  TIMESTAMP   NOT NULL DEFAULT now(),
    interaction_count BIGINT      NOT NULL DEFAULT 1,
    created_at        TIMESTAMP   NOT NULL DEFAULT now(),
    updated_at        TIMESTAMP   NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX index_organization_interactions_organization_id_user_id ON organization_interactions (organization_id, user_id);
