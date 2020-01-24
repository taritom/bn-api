ALTER TABLE orders
    ADD referrer TEXT;

UPDATE orders
SET referrer = CASE WHEN tracking_data ->> 'referrer' != '' THEN tracking_data ->> 'referrer' ELSE NULL END
WHERE tracking_data ->> 'referrer' IS NOT NULL;

UPDATE orders
SET source = coalesce(CASE WHEN tracking_data ->> 'utm_source' != '' THEN tracking_data ->> 'utm_source' ELSE NULL END,
                      source)
WHERE (tracking_data ->> 'utm_source')::TEXT <> source;

UPDATE orders
SET medium = coalesce(medium, 'referral'),
    source = coalesce(CASE
                          WHEN tracking_data ->> 'utm_source' != '' THEN tracking_data ->> 'utm_source'
                          ELSE replace(substring(tracking_data ->> 'referrer', '://((www\.)?[^/]*)/?'), 'www.', '') END,
                      source)
WHERE referrer IS NOT NULL;

UPDATE analytics_page_views
SET medium = 'referral',
    source = CASE
                 WHEN url ILIKE '%utm_source%' THEN source
                 ELSE replace(substring(referrer, '://((www\.)?[^/]*)/?'), 'www.', '') END
WHERE referrer != ''
  AND medium = '';

UPDATE analytics_page_views
SET source= 'facebook.com',
    medium=CASE WHEN medium = '' THEN 'referral' ELSE medium END
WHERE source = 'facebook';


UPDATE orders
SET source = 'facebook.com',
    medium = CASE WHEN coalesce(medium, '') = '' THEN 'referral' ELSE medium END
WHERE source = 'facebook';


ALTER TABLE analytics_page_views
    DROP CONSTRAINT analytics_page_views_unique;
--DROP INDEX analytics_page_views_unique;

ALTER TABLE analytics_page_views
    ADD CONSTRAINT analytics_page_views_unique UNIQUE (date, hour, event_id, source, medium, term, content, platform,
                                                       campaign, url, client_id,
                                                       user_agent, code, referrer);
