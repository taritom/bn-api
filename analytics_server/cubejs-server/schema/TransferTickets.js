cube(`TransferTickets`, {
  sql: `SELECT * FROM public.transfer_tickets`,

  joins: {
    Transfers: {
      sql: `${CUBE}.transfer_id = ${Transfers}.id`,
      relationship: `belongsTo`
    },
      TicketInstances: {
        sql:`${CUBE}.ticket_instance_id = ${TicketInstances}.id`,
          relationship:`belongsTo`
      }
  },

  measures: {
    count: {
      type: `count`,
      drillMembers: [id, ticketInstanceId, transferId, createdAt, updatedAt]
    }
  },

  dimensions: {
    id: {
      sql: `id`,
      type: `string`,
      primaryKey: true
    },

    ticketInstanceId: {
      sql: `ticket_instance_id`,
      type: `string`
    },

    transferId: {
      sql: `transfer_id`,
      type: `string`
    },

    createdAt: {
      sql: `created_at`,
      type: `time`
    },

    updatedAt: {
      sql: `updated_at`,
      type: `time`
    }
  }
});
