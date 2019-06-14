ALTER TABLE transfers
  ADD cancelled_by_user_id UUID NULL;
ALTER TABLE refunds
  ADD reason TEXT NULL;
