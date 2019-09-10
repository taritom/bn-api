

alter table domain_event_publishers
    add last_domain_event_seq BIGINT;
alter table domain_event_publishers
  add deleted_at TIMESTAMP;

alter table domain_events
    add seq BIGSERIAL;

alter table domain_events
    add organization_id Uuid REFERENCES organizations(id);

create index index_domain_events_seq on domain_events (seq);
create index index_domain_events_organization_id on domain_events(organization_id);
