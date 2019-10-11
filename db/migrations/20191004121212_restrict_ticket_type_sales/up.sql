alter table ticket_types add web_sales_enabled bool not null default true;
alter table ticket_types add box_office_sales_enabled bool not null default true;
alter table ticket_types add app_sales_enabled bool not null default true;
