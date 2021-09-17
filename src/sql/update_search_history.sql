UPDATE search_history
SET uses = uses + 1, modified = ?2
WHERE expression = ?1

