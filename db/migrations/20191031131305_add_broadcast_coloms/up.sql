alter table broadcasts
    add subject  TEXT,
    add audience varchar(100) not null default 'PeopleAtTheEvent';


alter table broadcasts
    drop name;
