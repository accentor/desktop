CREATE TABLE plays (
    id INTEGER PRIMARY KEY,
    played_at TEXT NOT NULL,
    track_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    loaded_at INTEGER NOT NULL
);
