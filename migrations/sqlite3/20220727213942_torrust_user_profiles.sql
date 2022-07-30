CREATE TABLE IF NOT EXISTS torrust_user_profiles (
    user_id INTEGER NOT NULL PRIMARY KEY,
    username TEXT NOT NULL,
    email TEXT,
    email_verified BOOL DEFAULT FALSE,
    bio TEXT,
    avatar TEXT,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id)
)
