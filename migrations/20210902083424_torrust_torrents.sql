CREATE TABLE IF NOT EXISTS torrust_torrents (
    torrent_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uploader_id INTEGER,
    title VARCHAR(256) NOT NULL,
    description TEXT,
    FOREIGN KEY(uploader_id) REFERENCES torrust_users(user_id)
)