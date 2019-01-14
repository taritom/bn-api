ALTER TABLE order_items ADD refunded_quantity bigint NOT NULL DEFAULT 0;

CREATE TABLE refunded_tickets (
    id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
    order_item_id uuid NOT NULL references order_items(id),
    ticket_instance_id uuid NOT NULL references ticket_instances(id),
    fee_refunded_at TIMESTAMP NULL,
    ticket_refunded_at TIMESTAMP NULL,
    created_at TIMESTAMP NOT NULL DEFAULT now(),
    updated_at TIMESTAMP NOT NULL DEFAULT now()
);

CREATE INDEX index_refunded_tickets_ticket_instance_id ON refunded_tickets(ticket_instance_id);
CREATE INDEX index_refunded_tickets_order_item_id ON refunded_tickets(order_item_id);
