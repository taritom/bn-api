ALTER TABLE refunds
    ADD manual_override BOOLEAN NOT NULL DEFAULT 'F';

ALTER TABLE events
    ADD settled_at TIMESTAMP NULL;

-- Accurately set settled_at for any where a settlement did exist
UPDATE events
SET settled_at = s.created_at
FROM events e
JOIN settlements s
ON e.organization_id = s.organization_id
AND e.event_end <= s.end_time
AND e.event_end >= s.start_time
WHERE e.id = events.id;

-- Update any ancient events from before automatic settlements
UPDATE events
SET settled_at = event_end
WHERE event_end <= (
  SELECT MAX(s.end_time)
  FROM settlements s
  WHERE s.organization_id = events.organization_id
  GROUP BY s.organization_id
)
AND settled_at IS NULL;
