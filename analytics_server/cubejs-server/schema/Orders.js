cube(`Orders`, {
  sql: `SELECT * FROM public.orders`,

  joins: {
    OrderItems: {
        sql: `${Orders}.id  = ${OrderItems}.order_id`,
        relationship: `hasMany`
    },
      OrderTicketTypes: {
          sql: `${OrderTicketTypes}.order_id = ${Orders}.id`,
          relationship: `hasMany`
      }

  },

  measures: {
    count: {
      type: `count`,
    },
      revenue: {
        type: `number`,
          sql: `${OrderItems.revenue}`,
          format: `currency`
      },

      faceValueRevenue: {
          type: `number`,
          sql: `${OrderItems.faceValueRevenue}`,
          format: `currency`,
      },
  },

  dimensions: {
    id: {
      sql: `id`,
      type: `string`,
      primaryKey: true,
        shown: true
    },

       number: {
        sql: `'#' || substring(${CUBE}.id::text, 29)`,
           type: `string`
       },


    userId: {
      sql: `user_id`,
      type: `string`
    },

    status: {
      sql: `status`,
      type: `string`
    },

    orderType: {
      sql: `order_type`,
      type: `string`
    },

    onBehalfOfUserId: {
      sql: `on_behalf_of_user_id`,
      type: `string`
    },

    boxOfficePricing: {
      sql: `box_office_pricing`,
      type: `string`
    },

    checkoutUrl: {
      sql: `checkout_url`,
      type: `string`
    },

    createUserAgent: {
      sql: `create_user_agent`,
      type: `string`
    },

    purchaseUserAgent: {
      sql: `purchase_user_agent`,
      type: `string`
    },

    externalPaymentType: {
      sql: `external_payment_type`,
      type: `string`
    },

    trackingData: {
      sql: `tracking_data`,
      type: `string`
    },

    source: {
      sql: `source`,
      type: `string`
    },

    campaign: {
      sql: `campaign`,
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

    settlementId: {
      sql: `settlement_id`,
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

    orderDate: {
      sql: `order_date`,
      type: `time`
    },

    expiresAt: {
      sql: `expires_at`,
      type: `time`
    },

    paidAt: {
      sql: `paid_at`,
      type: `time`
    },

    checkoutUrlExpires: {
      sql: `checkout_url_expires`,
      type: `time`
    }
  }
});
