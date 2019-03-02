CREATE TABLE IF NOT EXISTS aliases (
  name TEXT PRIMARY KEY,
  original TEXT,
  recursive BOOLEAN
);

