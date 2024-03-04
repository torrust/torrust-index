CREATE TABLE IF NOT EXISTS torrust_torrent_http_seeds (
    http_seed_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    torrent_id INTEGER NOT NULL,
    seed_url TEXT NOT NULL,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE
)
