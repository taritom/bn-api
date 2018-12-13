alter table organization_users
    rename column "role" to old_role;


alter table organization_users
    add "role" text[] not NULL;


update organization_users
set "role" = Array[old_role];


alter table organization_users
drop column old_role;

insert into organization_users (organization_id, user_id,  role)
select id, owner_user_id, ARRAY['OrgOwner']
from organizations;

alter table organizations
drop column owner_user_id;