DROP INDEX index_transfer_tickets_transfer_id_ticket_instance_id;

DROP INDEX index_events_organization_id_event_end;

DROP INDEX index_transfers_destination_user_id;

DROP INDEX index_orders_on_behalf_of_user_id;
CREATE INDEX index_orders_on_behalf_of_user_id ON orders (user_id);
