INSERT INTO tags
SELECT ?1, ?2, ?3
WHERE NOT EXISTS (
  SELECT 1 FROM tags
  WHERE tag = ?1 AND path = ?2 AND source = ?3
)
