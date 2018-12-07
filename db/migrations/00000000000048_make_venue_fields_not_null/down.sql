ALTER TABLE venues
    ALTER COLUMN region_id SET NULL;


ALTER TABLE venues
    DROP COLUMN google_place_id;

ALTER TABLE venues
    DROP COLUMN latitude;

ALTER TABLE venues
    DROP COLUMN longitude;

ALTER TABLE venues
    ALTER COLUMN address SET NULL;

ALTER TABLE venues
    ALTER COLUMN city SET NULL;

ALTER TABLE venues
    ALTER COLUMN country SET NULL;

ALTER TABLE venues
    ALTER COLUMN postal_code SET NULL;

ALTER TABLE venues
    ALTER COLUMN state SET NULL;


UPDATE venues
SET state       = CASE WHEN state = 'Unknown' THEN NULL ELSE state END,
    reGion_id   = CASE WHEN reGion_id = '00000000-0000-0000-0000-000000000000' THEN NULL ELSE region_id END,
    address     = CASE WHEN address = 'Unknown' THEN NULL ELSE address END,
    city= CASE WHEN city = 'Unknown' THEN NULL ELSE city END,
    country     = CASE WHEN country = 'Unknown' THEN NULL ELSE country END,
    postal_code = CASE WHEN postal_code = 'Unknown' THEN NULL ELSE postal_code END
;

DELETE
FROM regions
WHERE id = '00000000-0000-0000-0000-000000000000';
