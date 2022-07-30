CREATE TABLE IF NOT EXISTS torrust_tracker_keys (
    tracker_key_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    user_id INTEGER NOT NULL,
    tracker_key CHAR(32) NOT NULL,
    date_expiry DATETIME NOT NULL,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE
)
