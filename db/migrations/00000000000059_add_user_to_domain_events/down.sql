drop index index_domain_events_user_id;

alter table domain_events
drop column user_id;