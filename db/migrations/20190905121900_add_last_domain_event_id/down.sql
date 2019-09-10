
drop index index_domain_events_seq ;
drop index index_domain_events_organization_id;
alter table domain_event_publishers
    drop last_domain_event_seq;
alter table domain_event_publishers
    drop deleted_at ;

alter table domain_events
    drop seq;

alter table domain_event
  drop organization_id;
-- TODO: Populate this field


