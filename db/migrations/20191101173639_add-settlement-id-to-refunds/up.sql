ALTER TABLE refunds
  ADD settlement_id Uuid NULL references settlements(id);
CREATE INDEX index_refunds_settlement_id ON refunds (settlement_id);
