ALTER TABLE ticket_instances
  ADD check_in_source TEXT NULL;
CREATE INDEX index_ticket_instances_check_in_source ON ticket_instances (check_in_source);
