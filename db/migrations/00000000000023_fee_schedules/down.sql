DROP INDEX IF EXISTS index_organizations_fee_schedule_id;
ALTER TABLE organizations
  DROP COLUMN fee_schedule_id;
DROP INDEX IF EXISTS index_fee_schedules_name;
DROP TABLE IF EXISTS fee_schedules;
