CREATE TABLE codec_conversions (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    ffmpeg_params TEXT NOT NULL,
    resulting_codec_id INTEGER NOT NULL,
    loaded_at INTEGER NOT NULL
);
