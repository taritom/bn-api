alter table domain_events
    add user_id uuid  null references users(id);

create index index_domain_events_user_id on domain_events (user_id);