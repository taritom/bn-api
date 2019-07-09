DELETE FROM push_notification_tokens pnt USING (
    SELECT MIN(ctid) as ctid, user_id, token_source, token
    FROM push_notification_tokens
    GROUP BY user_id, token_source, token HAVING COUNT(*) > 1
) pnt_b
WHERE
        pnt.user_id = pnt_b.user_id
  AND pnt.token_source = pnt_b.token_source
  AND pnt.token = pnt_b.token
  AND pnt.ctid <> pnt_b.ctid;

-- Add a unique index
ALTER TABLE push_notification_tokens ADD CONSTRAINT unique_push_notification_token_per_source UNIQUE ( user_id, token_source, token);
