CREATE TABLE playlists (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    user_id INTEGER NOT NULL,
    access TEXT NOT NULL,
    playlist_type TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    loaded_at INTEGER NOT NULL
);

CREATE TABLE playlist_items (
    playlist_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    PRIMARY KEY (playlist_id, position)
);
