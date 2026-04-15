-- ADR-T-008: Drop the legacy `administrator` column.
-- SQLite < 3.35.0 does not support ALTER TABLE … DROP COLUMN,
-- so we use a table-rebuild approach.

CREATE TABLE IF NOT EXISTS torrust_users_new (
    user_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    date_registered TEXT,
    date_imported TEXT DEFAULT NULL,
    token_generation INTEGER NOT NULL DEFAULT 0,
    role TEXT NOT NULL DEFAULT 'registered'
);

INSERT INTO torrust_users_new (user_id, date_registered, date_imported, token_generation, role)
    SELECT user_id, date_registered, date_imported, token_generation, role FROM torrust_users;

DROP TABLE torrust_users;

ALTER TABLE torrust_users_new RENAME TO torrust_users;
