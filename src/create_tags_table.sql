CREATE TABLE IF NOT EXISTS tags (
  tag TEXT KEY,
  path TEXT,
  UNIQUE (tag, path)
);
