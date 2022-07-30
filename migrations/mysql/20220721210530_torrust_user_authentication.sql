CREATE TABLE IF NOT EXISTS torrust_user_authentication (
    user_id INTEGER NOT NULL PRIMARY KEY,
    password CHAR(32) NOT NULL,
    FOREIGN KEY(user_id) REFERENCES torrust_users(user_id)
)
