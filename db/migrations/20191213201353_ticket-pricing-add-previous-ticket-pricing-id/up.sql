ALTER TABLE ticket_pricing
    ADD previous_ticket_pricing_id Uuid NULL REFERENCES ticket_pricing(id);
