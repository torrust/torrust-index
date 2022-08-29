CREATE TABLE IF NOT EXISTS torrust_tracker_keys (
    tracker_key_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    tracker_key TEXT NOT NULL,
    date_expiry INTEGER NOT NULL,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)
