CREATE TABLE IF NOT EXISTS torrust_users_new (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    date_registered TEXT DEFAULT NULL,
    administrator BOOL NOT NULL DEFAULT FALSE
);

INSERT INTO torrust_users_new SELECT * FROM torrust_users;

DROP TABLE torrust_users;

ALTER TABLE torrust_users_new RENAME TO torrust_users