alter table organization_invites
    rename column "role" to old_role;

alter table organization_invites
    add "roles" text[] NULL;

update organization_invites
  set "roles" = Array[old_role];

alter table organization_invites
  alter column "roles" set NOT NULL;

alter table organization_invites
  drop column old_role;
