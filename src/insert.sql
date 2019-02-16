INSERT INTO images
SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9
WHERE (SELECT changes() = 0)
