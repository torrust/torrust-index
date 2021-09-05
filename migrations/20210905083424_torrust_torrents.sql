CREATE TABLE IF NOT EXISTS torrust_torrents (
    torrent_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    uploader_id INTEGER,
    info_hash VARCHAR(20),
    title VARCHAR(256) NOT NULL,
    category VARCHAR(32) NOT NULL,
    description TEXT,
    bencode TEXT,
    FOREIGN KEY(uploader_id) REFERENCES torrust_users(user_id)
)
