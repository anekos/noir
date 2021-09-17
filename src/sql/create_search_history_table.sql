CREATE TABLE IF NOT EXISTS search_history (
  expression TEXT PRIMARY KEY,
  uses INTEGER,
  created TEXT,
  modified TEXT
);
