ALTER TABLE orders
    DROP referrer;


ALTER TABLE analytics_page_views
    DROP CONSTRAINT analytics_page_views_unique;


ALTER TABLE analytics_page_views
    ADD CONSTRAINT analytics_page_views_unique UNIQUE (date, hour, event_id, source, medium, term, content, platform,
                                                       campaign, url, client_id,
                                                       user_agent, code);
