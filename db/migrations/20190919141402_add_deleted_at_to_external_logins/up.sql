ALTER TABLE external_logins
    ADD deleted_at TIMESTAMP;
DROP INDEX index_external_logins_external_user_id_site;

CREATE INDEX index_external_logins_external_user_id_site
    ON external_logins (external_user_id, site);

