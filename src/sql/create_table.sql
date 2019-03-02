CREATE TABLE IF NOT EXISTS images (
  path TEXT PRIMARY KEY,
  width INTEGER,
  height INTEGER,
  ratio_width INTEGER,
  ratio_height INTEGER,
  mime_type TEXT,
  animation BOOLEAN,
  file_size INTEGER,
  created TEXT,
  modified TEXT,
  accessed TEXT
);
CREATE TABLE IF NOT EXISTS tags (
  tag TEXT PRIMARY KEY,
  path TEXT
);
