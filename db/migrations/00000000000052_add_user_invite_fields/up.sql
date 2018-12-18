ALTER TABLE users
    ADD COLUMN accepted_terms_date TIMESTAMP NULL;
ALTER TABLE users
    ADD COLUMN invited_at TIMESTAMP NULL;
