alter table organization_invites
    add role text NULL;

update organization_invites
set role = 'OrgMember';

alter table organization_invites
alter column role set not null;