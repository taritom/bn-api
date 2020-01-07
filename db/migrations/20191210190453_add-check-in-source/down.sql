DROP INDEX IF EXISTS index_ticket_instances_check_in_source;
ALTER TABLE ticket_instances
  DROP check_in_source;
