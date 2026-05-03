CREATE TABLE auth_tokens (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    user_agent TEXT NOT NULL,
    application TEXT,
    loaded_at INTEGER NOT NULL
);
