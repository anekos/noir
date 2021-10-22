CREATE TABLE IF NOT EXISTS images (
  path TEXT PRIMARY KEY,
  width INTEGER,
  height INTEGER,
  ratio_width INTEGER,
  ratio_height INTEGER,
  format TEXT,
  animation BOOLEAN,
  file_size INTEGER,
  dhash TEXT,
  created TEXT,
  modified TEXT,
  accessed TEXT
);
