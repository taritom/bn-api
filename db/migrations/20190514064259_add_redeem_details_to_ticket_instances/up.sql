ALTER TABLE ticket_instances
    ADD COLUMN redeemed_by_user_id UUID,
    ADD COLUMN redeemed_at TIMESTAMP;
