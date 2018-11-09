ALTER TABLE order_items DROP CONSTRAINT order_items_code_id_max_uses_valid;
DROP FUNCTION order_items_code_id_max_uses_valid;

ALTER TABLE order_items DROP CONSTRAINT order_items_code_id_max_tickets_per_user_valid;
DROP FUNCTION order_items_code_id_max_tickets_per_user_valid;

DROP INDEX index_order_items_code_id;
ALTER TABLE order_items DROP COLUMN code_id;

DROP INDEX index_codes_redemption_code;
ALTER TABLE codes DROP CONSTRAINT codes_start_date_prior_to_end_date;

DROP INDEX index_codes_event_id_code_type;

DROP TABLE codes;
