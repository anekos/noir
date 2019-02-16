CREATE TABLE IF NOT EXISTS images (
  path TEXT PRIMARY KEY,
  width INTEGER,
  height INTEGER,
  ratio_width INTEGER,
  ratio_height INTEGER,
  mime_type TEXT,
  animation BOOLEAN,
  file_size INTEGER,
  file_extension TEXT
);
