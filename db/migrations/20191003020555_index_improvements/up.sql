DROP INDEX index_orders_on_behalf_of_user_id;
CREATE INDEX index_orders_on_behalf_of_user_id ON orders (on_behalf_of_user_id);

CREATE INDEX index_transfers_destination_user_id ON transfers(destination_user_id);

CREATE INDEX index_events_organization_id_event_end ON events (organization_id, event_end);

CREATE UNIQUE INDEX index_transfer_tickets_transfer_id_ticket_instance_id ON transfer_tickets(transfer_id, ticket_instance_id);
