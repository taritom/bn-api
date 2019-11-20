cube(`Organizations`, {
  sql: `SELECT * FROM public.organizations`,
  
  joins: {
    
  },
  
  measures: {
    count: {
      type: `count`,
      drillMembers: [id, name, city, country, sendgridApiKey, feeScheduleId, allowedPaymentProviders, slugId, createdAt, updatedAt]
    }
  },
  
  dimensions: {
    id: {
      sql: `id`,
      type: `string`,
      primaryKey: true
    },
    
    name: {
      sql: `name`,
      type: `string`
    },
    
    address: {
      sql: `address`,
      type: `string`
    },
    
    city: {
      sql: `city`,
      type: `string`
    },
    
    state: {
      sql: `state`,
      type: `string`
    },
    
    country: {
      sql: `country`,
      type: `string`
    },
    
    postalCode: {
      sql: `postal_code`,
      type: `string`
    },
    
    phone: {
      sql: `phone`,
      type: `string`
    },
    
    sendgridApiKey: {
      sql: `sendgrid_api_key`,
      type: `string`
    },
    
    googleGaKey: {
      sql: `google_ga_key`,
      type: `string`
    },
    
    facebookPixelKey: {
      sql: `facebook_pixel_key`,
      type: `string`
    },
    
    feeScheduleId: {
      sql: `fee_schedule_id`,
      type: `string`
    },
    
    allowedPaymentProviders: {
      sql: `allowed_payment_providers`,
      type: `string`
    },
    
    timezone: {
      sql: `timezone`,
      type: `string`
    },
    
    ccFeePercent: {
      sql: `cc_fee_percent`,
      type: `string`
    },
    
    globeeApiKey: {
      sql: `globee_api_key`,
      type: `string`
    },
    
    settlementType: {
      sql: `settlement_type`,
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
    
    updatedAt: {
      sql: `updated_at`,
      type: `time`
    }
  }
});
