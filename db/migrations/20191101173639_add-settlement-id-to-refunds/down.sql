DROP INDEX IF EXISTS index_refunds_settlement_id;
ALTER TABLE refunds
  DROP settlement_id;
