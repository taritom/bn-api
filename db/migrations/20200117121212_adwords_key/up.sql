ALTER TABLE organizations
    ADD google_ads_conversion_id TEXT;

ALTER TABLE organizations
    ADD google_ads_conversion_labels TEXT[] NOT NULL DEFAULT '{}';
