CREATE TABLE IF NOT EXISTS torrust_user_public_keys (
    public_key_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    public_key TEXT UNIQUE NOT NULL,
    date_registered TEXT NOT NULL,
    date_expiry TEXT NOT NULL,
    revoked INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)
