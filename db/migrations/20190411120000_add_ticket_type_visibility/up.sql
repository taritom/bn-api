ALTER TABLE ticket_types
    ADD visibility VARCHAR(100) NOT NULL DEFAULT 'Always';

UPDATE ticket_types
SET visibility = CASE
                     WHEN is_private THEN 'Hide'
                     WHEN sold_out_behavior = 'ShowSoldOut' THEN 'Always'
                     WHEN sold_out_behavior = 'Hide' THEN 'WhenAvailable'
                     ELSE 'Always' END
WHERE 1 = 1;

ALTER TABLE ticket_types
    DROP is_private;

ALTER TABLE ticket_types
    DROP sold_out_behavior;