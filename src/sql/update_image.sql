UPDATE images
SET width = ?2,
    height = ?3,
    ratio_width = ?4,
    ratio_height = ?5,
    mime_type = ?6,
    animation = ?7,
    file_size = ?8,
    created = ?9,
    modified = ?10,
    accessed = ?11
WHERE path = ?1
