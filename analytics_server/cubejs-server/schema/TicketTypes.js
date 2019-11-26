cube(`TicketTypes`, {
  sql: `SELECT * FROM public.ticket_types WHERE ${FILTER_PARAMS.TicketTypes.eventId.filter(JSON.stringify(USER_CONTEXT))}`,

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
      type: `count`,
      drillMembers: [id, eventId, name, parentId, endDateType, createdAt, updatedAt, startDate, endDate]
    }
  },

  dimensions: {
    id: {
      sql: `id`,
      type: `string`,
      primaryKey: true
    },

    eventId: {
      sql: `event_id`,
      type: `string`
    },

    name: {
      sql: `name`,
      type: `string`
    },

    description: {
      sql: `description`,
      type: `string`
    },

    status: {
      sql: `status`,
      type: `string`
    },

    parentId: {
      sql: `parent_id`,
      type: `string`
    },

    visibility: {
      sql: `visibility`,
      type: `string`
    },

    endDateType: {
      sql: `end_date_type`,
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
      type: `time`
    },

    updatedAt: {
      sql: `updated_at`,
      type: `time`
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

    deletedAt: {
      sql: `deleted_at`,
      type: `time`
    }
  }
});
