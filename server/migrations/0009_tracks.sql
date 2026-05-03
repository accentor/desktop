CREATE TABLE tracks (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    normalized_title TEXT NOT NULL,
    number INTEGER NOT NULL,
    album_id INTEGER NOT NULL,
    review_comment TEXT,
    codec_id INTEGER,
    length INTEGER,
    bitrate INTEGER,
    location_id INTEGER,
    audio_file_id INTEGER,
    filename TEXT,
    sample_rate INTEGER,
    bit_depth INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    loaded_at INTEGER NOT NULL
);

CREATE TABLE track_artists (
    track_id INTEGER NOT NULL,
    artist_id INTEGER NOT NULL,
    role TEXT NOT NULL,
    "order" INTEGER NOT NULL,
    name TEXT NOT NULL,
    normalized_name TEXT NOT NULL,
    hidden INTEGER NOT NULL,
    PRIMARY KEY (track_id, role, "order")
);

CREATE TABLE track_genres (
    track_id INTEGER NOT NULL,
    genre_id INTEGER NOT NULL,
    PRIMARY KEY (track_id, genre_id)
);
