CREATE TABLE event_users (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
  user_id UUID NOT NULL REFERENCES users(id),
  event_id UUID NOT NULL REFERENCES events(id),
  role TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT now(),
  updated_at TIMESTAMP NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX index_event_users_event_id_user_id ON event_users (event_id, user_id);

insert into event_users(user_id, event_id, role)
select ou.user_id, e.*, 'Promoter'
from organization_users ou
join unnest(ou.event_ids) e on 1=1
where 'Promoter' = ANY(ou.role);

insert into event_users(user_id, event_id, role)
select ou.user_id, e.*, 'PromoterReadOnly'
from organization_users ou
join unnest(ou.event_ids) e on 1=1
where 'PromoterReadOnly' = ANY(ou.role);

ALTER TABLE organization_users
  DROP COLUMN event_ids;
