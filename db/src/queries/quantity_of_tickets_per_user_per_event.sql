SELECT order_items.ticket_type_id,
       CAST(COUNT(ticket_instances.id) AS INTEGER) AS total_quantity
FROM orders
         INNER JOIN
     order_items ON (order_items.order_id = orders.id)
         INNER JOIN
     ticket_instances ON (ticket_instances.order_item_id = order_items.id)
WHERE ((orders.user_id = $1 AND orders.on_behalf_of_user_id IS NULL) OR orders.on_behalf_of_user_id = $1)
  AND order_items.event_id = $2
  AND order_items.ticket_type_id IS NOT NULL
  AND (ticket_instances.status = 'Purchased' OR
       (ticket_instances.status = 'Reserved' AND ticket_instances.reserved_until > now()))

GROUP BY order_items.event_id,
         order_items.ticket_type_id
