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
      type: `count`,
      drillMembers: [id, assetId, holdId, orderItemId, walletId, redeemedByUserId, firstNameOverride, lastNameOverride, createdAt, updatedAt]
    }
  },
  
  dimensions: {
    id: {
      sql: `id`,
      type: `string`,
      primaryKey: true
    },
    
    assetId: {
      sql: `asset_id`,
      type: `string`
    },
    
    holdId: {
      sql: `hold_id`,
      type: `string`
    },
    
    orderItemId: {
      sql: `order_item_id`,
      type: `string`
    },
    
    walletId: {
      sql: `wallet_id`,
      type: `string`
    },
    
    redeemKey: {
      sql: `redeem_key`,
      type: `string`
    },
    
    status: {
      sql: `status`,
      type: `string`
    },
    
    redeemedByUserId: {
      sql: `redeemed_by_user_id`,
      type: `string`
    },
    
    firstNameOverride: {
      sql: `first_name_override`,
      type: `string`
    },
    
    lastNameOverride: {
      sql: `last_name_override`,
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
    
    reservedUntil: {
      sql: `reserved_until`,
      type: `time`
    },
    
    redeemedAt: {
      sql: `redeemed_at`,
      type: `time`
    }
  }
});
