cube(`Events`, {
  sql: `SELECT * FROM public.events where ${USER_CONTEXT.event_id.filter("name")}`,

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
      type: `count`,
      drillMembers: [id, name, organizationId, venueId, videoUrl, overrideStatus, slugId, facebookEventId, createdAt, updatedAt, publishDate, redeemDate]
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

    organizationId: {
      sql: `organization_id`,
      type: `string`
    },

    venueId: {
      sql: `venue_id`,
      type: `string`
    },

    status: {
      sql: `status`,
      type: `string`
    },

    promoImageUrl: {
      sql: `promo_image_url`,
      type: `string`
    },

    additionalInfo: {
      sql: `additional_info`,
      type: `string`
    },

    ageLimit: {
      sql: `age_limit`,
      type: `string`
    },

    topLineInfo: {
      sql: `top_line_info`,
      type: `string`
    },

    videoUrl: {
      sql: `video_url`,
      type: `string`
    },

    isExternal: {
      sql: `is_external`,
      type: `string`
    },

    externalUrl: {
      sql: `external_url`,
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

    coverImageUrl: {
      sql: `cover_image_url`,
      type: `string`
    },

    privateAccessCode: {
      sql: `private_access_code`,
      type: `string`
    },

    facebookPixelKey: {
      sql: `facebook_pixel_key`,
      type: `string`
    },

    extraAdminData: {
      sql: `extra_admin_data`,
      type: `string`
    },

    slugId: {
      sql: `slug_id`,
      type: `string`
    },

    facebookEventId: {
      sql: `facebook_event_id`,
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
