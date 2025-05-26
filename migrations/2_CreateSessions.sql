CREATE TABLE IF NOT EXISTS sessions (
    session_key VARCHAR PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    expires_at VARCHAR NOT NULL
);