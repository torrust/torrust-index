CREATE TABLE IF NOT EXISTS torrust_torrents (
    torrent_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    uploader_id INTEGER NOT NULL,
    category_id INTEGER NOT NULL,
    info_hash CHAR(40) UNIQUE NOT NULL,
    size BIGINT NOT NULL,
    piece_length BIGINT NOT NULL,
    pieces TEXT NOT NULL,
    root_hash BOOLEAN NOT NULL DEFAULT FALSE,
    date_uploaded DATETIME NOT NULL,
    FOREIGN KEY(uploader_id) REFERENCES torrust_users(user_id) ON DELETE CASCADE,
    FOREIGN KEY(category_id) REFERENCES torrust_categories(category_id) ON DELETE CASCADE
)
