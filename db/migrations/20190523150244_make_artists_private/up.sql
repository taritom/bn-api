UPDATE artists
SET is_private = true, organization_id = a.organization_id
FROM (
  SELECT DISTINCT a.id, min(e.organization_id::text)::uuid as organization_id
  FROM artists a
  JOIN event_artists ea ON ea.artist_id = a.id
  JOIN events e ON e.id = ea.event_id
  WHERE a.is_private = false
  GROUP BY a.id
  HAVING count(DISTINCT e.organization_id) = 1
) a
WHERE artists.id = a.id;
