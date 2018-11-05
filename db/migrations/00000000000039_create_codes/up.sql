CREATE TABLE codes
(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    name TEXT NOT NULL,
    event_id uuid NOT NULL REFERENCES events(id),
    code_type TEXT NOT NULL,
    redemption_code TEXT NOT NULL CHECK (length(redemption_code) >= 6),
    max_uses BIGINT NOT NULL CHECK (max_uses > 0),
    discount_in_cents bigint NOT NULL CHECK (discount_in_cents > 0),
    start_date TIMESTAMP NOT NULL,
    end_date TIMESTAMP NOT NULL,
    max_tickets_per_user BIGINT NULL CHECK (coalesce (max_tickets_per_user, 1) >= 0),
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_codes_event_id_code_type ON codes (event_id, code_type);

ALTER TABLE codes ADD CONSTRAINT codes_start_date_prior_to_end_date CHECK (start_date < end_date);
CREATE UNIQUE INDEX index_codes_redemption_code ON codes(redemption_code);

ALTER TABLE ticket_instances ADD code_id UUID NULL REFERENCES codes (id);
CREATE INDEX index_ticket_instances_code_id ON ticket_instances (code_id);

ALTER TABLE orders ADD code_id UUID NULL REFERENCES codes (id);
CREATE INDEX index_orders_code_id ON orders (code_id);
