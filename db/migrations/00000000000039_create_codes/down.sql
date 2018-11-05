DROP INDEX index_ticket_instances_code_id;
ALTER TABLE ticket_instances DROP COLUMN code_id;

DROP INDEX index_orders_code_id;
ALTER TABLE orders DROP COLUMN code_id;

DROP INDEX index_codes_redemption_code;
ALTER TABLE codes DROP CONSTRAINT codes_start_date_prior_to_end_date;

DROP INDEX index_codes_event_id_code_type;

DROP TABLE codes;
