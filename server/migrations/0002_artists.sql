CREATE TABLE artists (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    normalized_name TEXT NOT NULL,
    review_comment TEXT,
    image TEXT,
    image100 TEXT,
    image250 TEXT,
    image500 TEXT,
    image_type TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    loaded_at INTEGER NOT NULL
);
