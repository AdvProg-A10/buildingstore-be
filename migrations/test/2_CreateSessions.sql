CREATE TABLE IF NOT EXISTS sessions (
    session_key TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    expires_at TEXT NOT NULL
);