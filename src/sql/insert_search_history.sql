INSERT INTO search_history
SELECT ?1, 1, ?2, ?2
WHERE NOT EXISTS (
  SELECT 1 FROM search_history
  WHERE expression = ?1
)
