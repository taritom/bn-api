cube(`Events`, {
  sql: `SELECT * FROM public.events`,

  joins: {
    Organizations: {
      sql: `${CUBE}.organization_id = ${Organizations}.id`,
      relationship: `belongsTo`
    },
      TicketTypes :{
        sql: `${Events}.id = ${TicketTypes}.event_id`,
          relationship:`hasMany`
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
    },

    name: {
      sql: `name`,
      type: `string`
    },



    status: {
      sql: `status`,
      type: `string`
    },


    ageLimit: {
      sql: `age_limit`,
      type: `string`
    },


    isExternal: {
      sql: `is_external`,
      type: `string`
    },


    overrideStatus: {
      sql: `override_status`,
      type: `string`
    },

    eventType: {
      sql: `event_type`,
      type: `string`
    },





    slugId: {
      sql: `slug_id`,
      type: `string`
    },

    createdAt: {
      sql: `created_at`,
      type: `time`
    },


    eventStart: {
      sql: `event_start`,
      type: `time`
    },

    doorTime: {
      sql: `door_time`,
      type: `time`
    },

    publishDate: {
      sql: `publish_date`,
      type: `time`
    },

    redeemDate: {
      sql: `redeem_date`,
      type: `time`
    },

    cancelledAt: {
      sql: `cancelled_at`,
      type: `time`
    },

    eventEnd: {
      sql: `event_end`,
      type: `time`
    },

    deletedAt: {
      sql: `deleted_at`,
      type: `time`
    }
  }
});
