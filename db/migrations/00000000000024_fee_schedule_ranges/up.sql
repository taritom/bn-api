CREATE TABLE fee_schedule_ranges (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  fee_schedule_id uuid not null REFERENCES fee_schedules(id),
  min_price bigint NOT NULL,
  fee bigint NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now()
);

-- Indices
CREATE UNIQUE INDEX index_fee_schedule_ranges_fee_schedule_id_min_price ON fee_schedule_ranges(fee_schedule_id, min_price)

