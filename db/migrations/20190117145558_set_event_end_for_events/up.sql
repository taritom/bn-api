UPDATE events SET event_end = event_start + interval '1 day' WHERE event_end IS NULL AND event_start IS NOT NULL;
