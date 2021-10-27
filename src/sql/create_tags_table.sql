CREATE TABLE IF NOT EXISTS tags (
  tag TEXT KEY,
  path TEXT,
  source TEXT,
  UNIQUE (tag, path, source)
);
