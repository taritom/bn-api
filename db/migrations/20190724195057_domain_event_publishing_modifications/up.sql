-- Remove ticket instance version of TransferTicketStarted as we're publishing that event type now
-- and the base transfer it belongs to has the event as well so it's duplicative.
delete from domain_events where main_table = 'TicketInstances' and event_type in ('TransferTicketStarted', 'TransferTicketCompleted', 'TransferTicketCancelled');

ALTER TABLE domain_events
    DROP COLUMN published_at;

CREATE TABLE domain_event_publishers (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  organization_id UUID NULL REFERENCES organizations(id),
  event_types TEXT[] NOT NULL,
  webhook_url TEXT NOT NULL,
  import_historic_events BOOLEAN NOT NULL DEFAULT 'F',
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE domain_event_published (
  domain_event_publisher_id UUID NOT NULL REFERENCES domain_event_publishers(id),
  domain_event_id UUID NOT NULL REFERENCES domain_events(id),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now(),
  PRIMARY KEY(domain_event_publisher_id, domain_event_id)
);

ALTER TABLE transfers
    ADD COLUMN direct BOOLEAN NOT NULL DEFAULT 'F';

CREATE TABLE temporary_users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  email TEXT NULL,
  phone TEXT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

ALTER TABLE transfers
    ADD COLUMN destination_temporary_user_id UUID NULL;

CREATE TABLE temporary_user_links (
  temporary_user_id UUID NOT NULL REFERENCES temporary_users(id),
  user_id UUID NOT NULL REFERENCES users(id),
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now(),
  PRIMARY KEY(temporary_user_id, user_id)
);
