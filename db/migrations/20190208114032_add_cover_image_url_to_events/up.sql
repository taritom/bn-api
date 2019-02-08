alter table events
    add cover_image_url text;

update events set cover_image_url = promo_image_url;

