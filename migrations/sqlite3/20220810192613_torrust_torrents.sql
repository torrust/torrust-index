CREATE TABLE IF NOT EXISTS torrust_torrents (
    torrent_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uploader_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    info_hash TEXT UNIQUE NOT NULL,
    size INTEGER NOT NULL,
    piece_length INTEGER NOT NULL,
    pieces TEXT NOT NULL,
    root_hash INT NOT NULL DEFAULT 0,
    date_uploaded TEXT NOT NULL,
    FOREIGN KEY(uploader_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE,
    FOREIGN KEY(category_id) REFERENCES torrust_categories(category_id) ON DELETE CASCADE
)
