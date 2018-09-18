CREATE TABLE fee_schedules (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  name TEXT NOT NULL,
  version SMALLINT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE UNIQUE INDEX index_fee_schedules_name ON fee_schedules(name, version);

ALTER TABLE organizations
  ADD fee_schedule_id uuid NOT NULL REFERENCES fee_schedules (id);

CREATE INDEX index_organizations_fee_schedule_id ON organizations (fee_schedule_id);
