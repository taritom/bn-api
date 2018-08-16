-- Define the organization_invites table
CREATE TABLE organization_invites (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  organization_id uuid NOT NULL REFERENCES organizations (id),
  inviter_id uuid NOT NULL REFERENCES users (id),
  user_email TEXT NOT NULL,
  create_at TIMESTAMP NOT NULL DEFAULT now(),
  security_token uuid UNIQUE,
  user_id uuid REFERENCES users (id),
  status_change_at TIMESTAMP ,
  accepted SMALLINT
);

-- Indices
CREATE INDEX index_organization_invites_organization_id ON organization_invites (organization_id);
CREATE INDEX index_organization_invites_user_id ON organization_invites (inviter_id);
CREATE INDEX index_organization_invitee_user_id ON organization_invites (user_id);
CREATE INDEX security_token ON organization_invites (security_token);