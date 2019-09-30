ALTER TABLE external_logins
    DROP deleted_at;
DROP INDEX index_external_logins_external_user_id_site;

CREATE UNIQUE INDEX index_external_logins_external_user_id_site
    ON external_logins (external_user_id, site);
