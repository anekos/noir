CREATE INDEX IF NOT EXISTS tags_index_path ON tags(tag, path);
CREATE INDEX IF NOT EXISTS tags_index_path_source ON tags(tag, path, source);
