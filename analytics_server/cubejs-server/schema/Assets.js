cube(`Assets`, {
  sql: `SELECT * FROM public.assets`,
  
  joins: {
    TicketTypes: {
      sql: `${CUBE}.ticket_type_id = ${TicketTypes}.id`,
      relationship: `belongsTo`
    }
  },
  
  measures: {
    count: {
      type: `count`,
      drillMembers: [id, ticketTypeId, blockchainName, blockchainAssetId, createdAt, updatedAt]
    }
  },
  
  dimensions: {
    id: {
      sql: `id`,
      type: `string`,
      primaryKey: true
    },
    
    ticketTypeId: {
      sql: `ticket_type_id`,
      type: `string`
    },
    
    blockchainName: {
      sql: `blockchain_name`,
      type: `string`
    },
    
    blockchainAssetId: {
      sql: `blockchain_asset_id`,
      type: `string`
    },
    
    status: {
      sql: `status`,
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
