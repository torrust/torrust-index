CREATE TABLE IF NOT EXISTS torrust_torrents (
    torrent_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uploader_id INTEGER NOT NULL,
    info_hash VARCHAR(20),
    title VARCHAR(256) NOT NULL,
    category_id INTEGER NOT NULL,
    description TEXT,
    FOREIGN KEY(uploader_id) REFERENCES torrust_users(user_id),
    FOREIGN KEY(category_id) REFERENCES torrust_categories(category_id)
)
