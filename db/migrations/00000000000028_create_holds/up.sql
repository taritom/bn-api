CREATE TABLE holds
(
    id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    name TEXT NOT NULL,
    -- Intentionally not a cascading delete:
    parent_hold_id uuid NULL REFERENCES holds(id),
    event_id uuid NOT NULL REFERENCES events(id),
    redemption_code TEXT NOT NULL,
    discount_in_cents bigint NULL CHECK (hold_type = 'Comp' or discount_in_cents >= 0),
    end_at TIMESTAMP NULL,
    max_per_order BIGINT NULL CHECK (coalesce (max_per_order, 10) >= 0),
    hold_type TEXT NOT NULL,
    ticket_type_id uuid NOT NULL REFERENCES ticket_types(id),
    email text null,
    phone text null,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX index_holds_event_id_name ON holds (
    event_id,
    parent_hold_id,
    name
);

CREATE UNIQUE INDEX index_holds_redemption_code ON holds(redemption_code);
CREATE INDEX index_holds_ticket_type_id ON holds(ticket_type_id);
CREATE INDEX index_holds_hold_type ON holds(hold_type);
