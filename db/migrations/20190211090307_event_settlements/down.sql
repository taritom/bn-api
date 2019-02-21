-- Settlements
DROP INDEX IF EXISTS index_settlements_organization_id;
DROP TABLE IF EXISTS settlements;

-- Settlement Transactions
DROP INDEX IF EXISTS index_settlement_transactions_settlement_id;
DROP INDEX IF EXISTS index_settlement_transactions_event_id;
DROP TABLE IF EXISTS settlement_transactions;
