cube(`OrderItems`, {
  sql: `SELECT * FROM public.order_items `,

  joins: {
    Orders: {
      sql: `${CUBE}.order_id = ${Orders}.id`,
      relationship: `belongsTo`
    },

    // TicketTypes: {
    //   sql: `${CUBE}.ticket_type_id = ${TicketTypes}.id`,
    //   relationship: `belongsTo`
    // },


  },

  measures: {
    count: {
      type: `count`,
      drillMembers: [id, orderId, ticketTypeId,  ticketPricingId, feeScheduleRangeId, parentId, holdId, codeId, createdAt, updatedAt]
    },

    quantity: {
      sql: `quantity`,
      type: `sum`
    },

    refundedQuantity: {
      sql: `refunded_quantity`,
      type: `sum`
    },
    ticketCount: {
      sql: `quantity - refunded_quantity`,
      type: `sum`,
      filters: [
          {sql: `${OrderItems}.item_type = 'Tickets'` }
      ],
    },
    revenue: {
        sql: `Round((quantity - refunded_quantity) * unit_price_in_cents / 100.0,2)`,
        type: `sum`,
        format: `currency`
    },
      faceValueRevenue: {
          sql: `Round((quantity - refunded_quantity) * unit_price_in_cents / 100.0,2)`,
          type: `sum`,
          format: `currency`,
          filters: [
              {sql: `${CUBE}.item_type NOT IN ('PerUnitFees', 'EventFees','CreditCardFees')`}
          ]
      }
  },

  dimensions: {
    id: {
      sql: `id`,
      type: `string`,
      primaryKey: true,

    },

    orderId: {
      sql: `order_id`,
      type: `string`
    },

    itemType: {
      sql: `item_type`,
      type: `string`
    },

    ticketTypeId: {
      sql: `ticket_type_id`,
      type: `string`
    },


    ticketPricingId: {
      sql: `ticket_pricing_id`,
      type: `string`
    },

    feeScheduleRangeId: {
      sql: `fee_schedule_range_id`,
      type: `string`
    },

    parentId: {
      sql: `parent_id`,
      type: `string`
    },

    holdId: {
      sql: `hold_id`,
      type: `string`
    },

    codeId: {
      sql: `code_id`,
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
