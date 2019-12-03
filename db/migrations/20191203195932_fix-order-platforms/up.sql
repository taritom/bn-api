update orders set platform = 'BoxOffice' where on_behalf_of_user_id is not null;
update orders set platform = 'Web' where platform is null;
