cube(`TicketInstances`, {
  sql: `SELECT * FROM public.ticket_instances`,

  joins: {
    Assets: {
      sql: `${CUBE}.asset_id = ${Assets}.id`,
      relationship: `belongsTo`
    },

    OrderItems: {
      sql: `${CUBE}.order_item_id = ${OrderItems}.id`,
      relationship: `belongsTo`
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
        shown: false
    },


    status: {
      sql: `status`,
      type: `string`
    },

    createdAt: {
      sql: `created_at`,
      type: `time`,
        title: "First Created"
    },


    redeemedAt: {
      sql: `redeemed_at`,
      type: `time`,
        title: `Redeemed Date`
    },

    eventId: {
        sql: `${Assets.eventId}`,
        type: `string`
    }

  }
});
