INSERT INTO aliases
SELECT ?1, ?2, ?3
WHERE (SELECT changes() = 0)
