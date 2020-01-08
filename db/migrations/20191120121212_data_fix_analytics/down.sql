alter table analytics_page_views
    alter column event_id type text using event_id::text;
