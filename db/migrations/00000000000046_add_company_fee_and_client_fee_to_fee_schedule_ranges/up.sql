ALTER TABLE fee_schedule_ranges
  ADD company_fee_in_cents bigint NOT NULL,
  ADD client_fee_in_cents bigint NOT NULL;
