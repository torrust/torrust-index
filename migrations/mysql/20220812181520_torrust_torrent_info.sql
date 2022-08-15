CREATE TABLE IF NOT EXISTS torrust_torrent_info (
    torrent_id INTEGER NOT NULL PRIMARY KEY,
    title VARCHAR(256) UNIQUE NOT NULL,
    description TEXT DEFAULT NULL,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE
)
