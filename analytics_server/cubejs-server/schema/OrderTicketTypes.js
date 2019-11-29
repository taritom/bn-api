cube(`OrderTicketTypes`, {
    sql: `SELECT DISTINCT order_items.id, orders.id as order_id, order_items.ticket_type_id from public.orders inner join public.order_items on orders.id = order_items.order_id WHERE order_items.ticket_type_id IS NOT NULL`
,
    joins: {
        TicketTypes: {
            relationship: `hasMany`,
            sql: `${OrderTicketTypes}.ticket_type_id = ${TicketTypes}.id`
        },
        Orders:{
            relationship: `hasMany`,
            sql: `${OrderTicketTypes}.order_id = ${Orders}.id`
        }
    },
    measures: {
        count: {
            type: `count`
        }
    },
    dimensions: {
        id: {
            sql: `id`,
            type: `string`,
            primaryKey: true,
            shown: true
        }
    }
});
