UPDATE images
SET width = ?2,
    height = ?3,
    ratio_width = ?4,
    ratio_height = ?5,
    format = ?6,
    animation = ?7,
    file_size = ?8,
    dhash = ?9,
    created = ?10,
    modified = ?11,
    accessed = ?12
WHERE path = ?1
