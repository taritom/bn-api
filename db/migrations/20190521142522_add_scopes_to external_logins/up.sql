alter table external_logins add scopes text[];

update external_logins set scopes = '{email}';


alter table external_logins alter column scopes set not null;