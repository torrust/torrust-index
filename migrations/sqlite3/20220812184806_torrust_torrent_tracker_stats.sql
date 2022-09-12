CREATE TABLE IF NOT EXISTS torrust_torrent_tracker_stats (
    torrent_id INTEGER NOT NULL PRIMARY KEY,
    tracker_url VARCHAR(256) NOT NULL,
    seeders INTEGER NOT NULL DEFAULT 0,
    leechers INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY(torrent_id) REFERENCES torrust_torrents(torrent_id) ON DELETE CASCADE,
    UNIQUE(torrent_id, tracker_url)
)
