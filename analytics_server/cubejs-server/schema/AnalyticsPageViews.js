cube(`AnalyticsPageViews`, {
  sql: `SELECT * FROM public.analytics_page_views`,

  joins: {
      Sources: {
        sql: `${CUBE}.source = ${Sources}.source`,
          relationship: 'belongsTo'
      }
  },

  measures: {
    count: {
      sql: `count`,
      type: `sum`
    },
    uniqueCount: {
        sql: `client_id`,
        type: `countDistinct`
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

    source: {
      sql: `source`,
      type: `string`
    },

    medium: {
      sql: `medium`,
      type: `string`
    },

    term: {
      sql: `term`,
      type: `string`
    },

    content: {
      sql: `content`,
      type: `string`
    },

    platform: {
      sql: `platform`,
      type: `string`
    },

    campaign: {
      sql: `campaign`,
      type: `string`
    },

    url: {
      sql: `url`,
      type: `string`
    },

    code: {
      sql: `code`,
      type: `string`
    },

    clientId: {
      sql: `client_id`,
      type: `string`
    },

    userAgent: {
      sql: `user_agent`,
      type: `string`
    },

    ipAddress: {
      sql: `ip_address`,
      type: `string`
    },

    date: {
      sql: `date`,
      type: `time`
    },

    hour: {
      sql: `hour`,
      type: `time`
    }
  }
});
