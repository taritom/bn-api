alter table broadcasts
    add sent_quantity BIGINT DEFAULT 0 NOT NULL,
    add opened_quantity BIGINT DEFAULT 0 NOT NULL;
