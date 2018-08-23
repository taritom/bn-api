
INSERT INTO public.users (id, first_name, last_name, email, phone, hashed_pw, password_modified_at, created_at, last_used, active, role, password_reset_token, password_reset_requested_at) VALUES ('4d72ddc1-fabf-4ac3-af9d-d6267ff57924', 'Mike', 'Berry', 'mike@tari.com', '555', '$argon2i$m=4096,t=3,p=1$akJ0Y1lJU0hWSVR2dWdyT3NoWWtBYlVjQ1doTlFHN3A$cfrrhqCbmmEtOC5/kOi7DC6LKrfTJ1YYeKM/k/CezWo', '2018-08-22 10:01:59.338182', '2018-08-22 10:01:59.338182', null, true, '{Guest, Admin}', null, null);

INSERT INTO public.organizations (id, owner_user_id, name, address, city, state, country, zip, phone) VALUES ('ac1e48f2-6765-4a18-b43c-d3c9836bc4c3', '4d72ddc1-fabf-4ac3-af9d-d6267ff57924', 'Jazzy', null, null, null, null, null, null);

INSERT INTO public.venues (id, name, address, city, state, country, zip, phone) VALUES ('0eb7fa9d-6a80-4c21-ac5c-d0682ab7dae6', 'Test venue', null, null, null, null, null, null);
INSERT INTO public.venues (id, name, address, city, state, country, zip, phone) VALUES ('bd24baee-c074-46a7-b5c9-8bdfefb10ef5', 'Test venue', null, null, null, null, null, null);

INSERT INTO public.events (id, name, organization_id, venue_id, created_at, ticket_sell_date, event_start) VALUES ('0b03f1f2-84b7-4899-b226-7fc841f4b054', 'Test Event', 'ac1e48f2-6765-4a18-b43c-d3c9836bc4c3', 'bd24baee-c074-46a7-b5c9-8bdfefb10ef5', '2018-08-22 10:44:40.330646', '2018-08-22 10:44:40.330646', '2018-11-12 12:00:00.000000');
INSERT INTO public.events (id, name, organization_id, venue_id, created_at, ticket_sell_date, event_start) VALUES ('1b20f925-d5c1-456d-9f59-e001f106a9c0', 'Test Event 2', 'ac1e48f2-6765-4a18-b43c-d3c9836bc4c3', 'bd24baee-c074-46a7-b5c9-8bdfefb10ef5', '2018-08-22 10:44:44.432111', '2018-08-22 10:44:44.432111', '2018-11-12 12:00:00.000000');
INSERT INTO public.events (id, name, organization_id, venue_id, created_at, ticket_sell_date, event_start) VALUES ('7d93995b-64ce-45d3-840d-9320ad185438', 'Test Event 4', 'ac1e48f2-6765-4a18-b43c-d3c9836bc4c3', 'bd24baee-c074-46a7-b5c9-8bdfefb10ef5', '2018-08-22 10:44:52.524310', '2018-08-22 10:44:52.524310', '2018-11-13 12:00:00.000000');



INSERT INTO public.ticket_allocations (id, event_id, tari_asset_id, created_on, synced_on, ticket_delta) VALUES ('46c10491-5596-4326-a7a3-0dc301ce4a0f', '7d93995b-64ce-45d3-840d-9320ad185438', 'Test 1', '2018-08-22 11:41:07.626539', '2018-08-22 11:41:07.619112', 100);
INSERT INTO public.ticket_allocations (id, event_id, tari_asset_id, created_on, synced_on, ticket_delta) VALUES ('cdf009a7-98e0-49d2-b84b-f6525b271ec2', '7d93995b-64ce-45d3-840d-9320ad185438', 'Test 1', '2018-08-22 11:41:10.016970', '2018-08-22 11:41:09.999551', 100);