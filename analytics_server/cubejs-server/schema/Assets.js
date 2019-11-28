cube(`Assets`, {
  // sql: `SELECT distinct assets.* FROM public.assets inner join ${TicketTypes.sql()} tt on assets.ticket_type_id = tt.id`,
    sql: `SELECT * FROM public.assets`,

  joins: {
    TicketTypes: {
      sql: `${CUBE}.ticket_type_id = ${TicketTypes}.id`,
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
        title: `First Created`
    },

    eventId: {
        sql: `${TicketTypes.eventId}`,
        type: `string`
    }
  }
});
