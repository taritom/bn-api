insert into domain_events(event_type, display_text, main_table, main_id, created_at, updated_at, user_id)
select 'PushNotificationTokenCreated', 'Push notification created', 'PushNotificationTokens', pnt.id, pnt.created_at, pnt.created_at, pnt.user_id
from push_notification_tokens pnt
left join domain_events de on de.main_table = 'PushNotificationTokens' and de.main_id = pnt.id and event_type = 'PushNotificationTokenCreated'
where de.id is null;
