CREATE TABLE IF NOT EXISTS torrust_user_public_keys (
    public_key_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    user_id INTEGER NOT NULL,
    public_key CHAR(32) UNIQUE NOT NULL,
    date_registered DATETIME NOT NULL,
    date_expiry DATETIME NOT NULL,
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id)
)
