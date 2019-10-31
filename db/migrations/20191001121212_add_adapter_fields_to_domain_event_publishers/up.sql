alter table domain_event_publishers
    add adapter varchar(100);
alter table domain_event_publishers
  add adapter_config jsonb;
