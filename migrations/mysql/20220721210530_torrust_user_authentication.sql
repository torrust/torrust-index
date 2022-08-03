CREATE TABLE IF NOT EXISTS torrust_user_authentication (
    user_id INTEGER NOT NULL PRIMARY KEY,
    password_hash TEXT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)
