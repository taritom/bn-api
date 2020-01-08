alter table analytics_page_views
alter column event_id type uuid using event_id::uuid;

alter table analytics_page_views
add created_at TIMESTAMP not null default now();

alter table analytics_page_views
add updated_at TIMESTAMP not null default now();

alter table analytics_page_views
add referrer TEXT not null default '';

update analytics_page_views
set source = 'facebook'
where url like '%fbclid%';

update analytics_page_views
set source = coalesce(substring(url from 'utm_source=([^&]*)&?'),'')
where url like '%utm_source%';

update analytics_page_views
set medium = coalesce(substring(url from 'utm_medium=([^&]*)&?'),'')
where url like '%utm_medium%';


update analytics_page_views
set term = coalesce(substring(url from 'utm_term=([^&]*)&?'),'')
where url like '%utm_term%';


update analytics_page_views
set content = coalesce(substring(url from 'utm_content=([^&]*)&?'),'')
where url like '%utm_content%';


update analytics_page_views
set campaign = coalesce(substring(url from 'utm_campaign=([^&]*)&?'),'')
where url like '%utm_campaign%';

update analytics_page_views
set platform = 'Web'
where user_agent like '%Mozilla%';

