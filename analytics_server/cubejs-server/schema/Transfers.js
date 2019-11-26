cube(`Transfers`, {
  sql: `SELECT * FROM public.transfers`,
  
  joins: {
    
  },
  
  measures: {
    count: {
      type: `count`,
      drillMembers: [id, sourceUserId, destinationUserId, cancelledByUserId, destinationTemporaryUserId, createdAt, updatedAt]
    }
  },
  
  dimensions: {
    id: {
      sql: `id`,
      type: `string`,
      primaryKey: true
    },
    
    sourceUserId: {
      sql: `source_user_id`,
      type: `string`
    },
    
    destinationUserId: {
      sql: `destination_user_id`,
      type: `string`
    },
    
    transferKey: {
      sql: `transfer_key`,
      type: `string`
    },
    
    status: {
      sql: `status`,
      type: `string`
    },
    
    transferMessageType: {
      sql: `transfer_message_type`,
      type: `string`
    },
    
    transferAddress: {
      sql: `transfer_address`,
      type: `string`
    },
    
    cancelledByUserId: {
      sql: `cancelled_by_user_id`,
      type: `string`
    },
    
    direct: {
      sql: `direct`,
      type: `string`
    },
    
    destinationTemporaryUserId: {
      sql: `destination_temporary_user_id`,
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
