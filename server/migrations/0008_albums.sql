CREATE TABLE albums (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    normalized_title TEXT NOT NULL,
    release TEXT NOT NULL,
    review_comment TEXT,
    edition TEXT,
    edition_description TEXT,
    image TEXT,
    image100 TEXT,
    image250 TEXT,
    image500 TEXT,
    image_type TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    loaded_at INTEGER NOT NULL
);

CREATE TABLE album_artists (
    album_id INTEGER NOT NULL,
    artist_id INTEGER NOT NULL,
    name TEXT NOT NULL,
    normalized_name TEXT NOT NULL,
    "order" INTEGER NOT NULL,
    separator TEXT,
    PRIMARY KEY (album_id, "order")
);

CREATE TABLE album_labels (
    album_id INTEGER NOT NULL,
    label_id INTEGER NOT NULL,
    catalogue_number TEXT,
    PRIMARY KEY (album_id, label_id)
);
