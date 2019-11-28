cube(`TicketTypes`, {
  sql: `SELECT * FROM public.ticket_types WHERE deleted_at IS NULL`,
    // sql: `SELECT * FROM public.ticket_types WHERE ${USER_CONTEXT.event_id.filter("event_id")} AND deleted_at IS NULL`,

  joins: {
    Events: {
      sql: `${CUBE}.event_id = ${Events}.id`,
      relationship: `belongsTo`
    },
      Assets:{
        sql: `${Assets}.ticket_type_id = ${TicketType}.id`,
          relationship: `hasMany`
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
        shown: `false`
    },

    name: {
      sql: `name`,
      type: `string`
    },

    status: {
      sql: `status`,
      type: `string`
    },

    visibility: {
      sql: `visibility`,
      type: `string`
    },

    webSalesEnabled: {
      sql: `web_sales_enabled`,
      type: `string`
    },

    boxOfficeSalesEnabled: {
      sql: `box_office_sales_enabled`,
      type: `string`
    },

    appSalesEnabled: {
      sql: `app_sales_enabled`,
      type: `string`
    },

    createdAt: {
      sql: `created_at`,
      type: `time`,
        title: `First Created`
    },

    startDate: {
      sql: `start_date`,
      type: `time`
    },

    endDate: {
      sql: `end_date`,
      type: `time`
    },

    cancelledAt: {
      sql: `cancelled_at`,
      type: `time`
    },
  }
});
