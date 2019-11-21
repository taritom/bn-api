cube(`Sources`, {
    sql: `SELECT DISTINCT source
          FROM public.analytics_page_views`,
    joins: {
        AnalyticsPageViews: {
            relationship: `hasMany`,
            sql: `${AnalyticsPageViews}.source = ${Sources}.source`
        },
        Orders: {
            relationship: `hasMany`,
            sql: `${Orders}.source = ${Sources}.source`
        }
    }
});
