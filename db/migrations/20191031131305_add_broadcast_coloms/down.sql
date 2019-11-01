alter table broadcasts
    drop subject,
    drop audience;

alter table broadcasts
    add name text not null;