CREATE TABLE IF NOT EXISTS sessions (
    session_key VARCHAR PRIMARY KEY,
    user_id INTEGER NOT NULL,
    expires_at VARCHAR NOT NULL
);