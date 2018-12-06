INSERT INTO regions(id, name)
VALUES ('00000000-0000-0000-0000-000000000000', 'Other');

UPDATE venueS
SET state       = COALESCE(state, 'Unknown'),
    region_id   = COALESCE(reGion_id, '00000000-0000-0000-0000-000000000000'),
    address     = COALESCE(address, 'Unknown'),
    city= coalesce(city, 'Unknown'),
    country     = coalesce(country, 'Unknown'),
    postal_code = coalesce(postal_code, 'Unknown')
;

ALTER TABLE venues
    ALTER COLUMN region_id SET NOT NULL;


ALTER TABLE venues
    ADD google_place_id TEXT NULL;

ALTER TABLE venues
    ADD latitude DOUBLE PRECISION NULL;

ALTER TABLE venues
    ADD longitude DOUBLE PRECISION NULL;

ALTER TABLE venues
    ALTER COLUMN address SET NOT NULL;

ALTER TABLE venues
    ALTER COLUMN city SET NOT NULL;

ALTER TABLE venues
    ALTER COLUMN country SET NOT NULL;

ALTER TABLE venues
    ALTER COLUMN postal_code SET NOT NULL;

ALTER TABLE venues
    ALTER COLUMN state SET NOT NULL;
