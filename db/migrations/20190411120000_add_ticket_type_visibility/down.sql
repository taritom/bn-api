ALTER TABLE ticket_types
    ADD is_private BOOL NULL;

ALTER TABLE ticket_types
    ADD sold_out_behavior TEXT NULL;

UPDATE ticket_types
SET sold_out_behavior = CASE
                            WHEN visibility = 'Always' THEN 'ShowSoldOut'
                            ELSE 'Hide' END,
    is_private        = visibility = 'Hidden'
WHERE 1 = 1;


ALTER TABLE ticket_types
    DROP visibility;
